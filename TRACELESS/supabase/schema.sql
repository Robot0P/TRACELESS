-- Traceless 许可证系统 Supabase 数据库结构
-- 创建日期: 2024-12-24
-- 版本: 1.0

-- =============================================
-- 1. 许可证表 (licenses)
-- =============================================
CREATE TABLE IF NOT EXISTS licenses (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    license_key VARCHAR(29) UNIQUE NOT NULL, -- 格式: XXXXX-XXXXX-XXXXX-XXXXX-XXXXX
    tier INTEGER NOT NULL DEFAULT 0,          -- 0:Free, 1:Monthly, 2:Quarterly, 3:Yearly
    machine_id VARCHAR(64),                   -- 绑定的机器ID (激活后填充)
    os_info VARCHAR(100),                     -- 操作系统信息 (激活后填充)
    os_version VARCHAR(50),                   -- 操作系统版本
    hostname VARCHAR(100),                    -- 主机名
    status VARCHAR(20) DEFAULT 'unused',      -- unused, active, expired, revoked
    created_at TIMESTAMPTZ DEFAULT NOW(),
    activated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    last_verified_at TIMESTAMPTZ,             -- 最后验证时间
    activation_count INTEGER DEFAULT 0,       -- 激活次数
    max_activations INTEGER DEFAULT 1,        -- 最大激活次数
    notes TEXT,                               -- 备注
    created_by UUID REFERENCES auth.users(id) -- 创建者 (管理员)
);

-- =============================================
-- 2. 激活记录表 (activation_logs)
-- =============================================
CREATE TABLE IF NOT EXISTS activation_logs (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    license_id UUID REFERENCES licenses(id) ON DELETE CASCADE,
    license_key VARCHAR(29) NOT NULL,
    machine_id VARCHAR(64) NOT NULL,
    os_info VARCHAR(100),                     -- 操作系统信息
    os_version VARCHAR(50),                   -- 操作系统版本
    hostname VARCHAR(100),                    -- 主机名
    action VARCHAR(20) NOT NULL,              -- activate, deactivate, verify, reject
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN DEFAULT true,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================
-- 3. 许可证类型配置表 (license_tiers)
-- =============================================
CREATE TABLE IF NOT EXISTS license_tiers (
    id INTEGER PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    duration_days INTEGER NOT NULL,           -- 有效期天数
    price DECIMAL(10, 2),
    features JSONB,                           -- 功能列表
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 插入默认许可证类型
INSERT INTO license_tiers (id, name, display_name, duration_days, price, features) VALUES
(0, 'free', '免费版', 0, 0, '{"scan": true, "file_shredder": true, "timestamp_modifier": true, "anti_analysis": true}'),
(1, 'monthly', '月度订阅', 30, 9.99, '{"scan": true, "file_shredder": true, "timestamp_modifier": true, "anti_analysis": true, "system_logs": true, "memory_cleanup": true, "network_cleanup": true, "registry_cleanup": true, "disk_encryption": true, "scheduled_tasks": true}'),
(2, 'quarterly', '季度订阅', 90, 24.99, '{"scan": true, "file_shredder": true, "timestamp_modifier": true, "anti_analysis": true, "system_logs": true, "memory_cleanup": true, "network_cleanup": true, "registry_cleanup": true, "disk_encryption": true, "scheduled_tasks": true}'),
(3, 'yearly', '年度订阅', 365, 79.99, '{"scan": true, "file_shredder": true, "timestamp_modifier": true, "anti_analysis": true, "system_logs": true, "memory_cleanup": true, "network_cleanup": true, "registry_cleanup": true, "disk_encryption": true, "scheduled_tasks": true}')
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    display_name = EXCLUDED.display_name,
    duration_days = EXCLUDED.duration_days,
    price = EXCLUDED.price,
    features = EXCLUDED.features;

-- =============================================
-- 4. 索引
-- =============================================
CREATE INDEX IF NOT EXISTS idx_licenses_license_key ON licenses(license_key);
CREATE INDEX IF NOT EXISTS idx_licenses_machine_id ON licenses(machine_id);
CREATE INDEX IF NOT EXISTS idx_licenses_status ON licenses(status);
CREATE INDEX IF NOT EXISTS idx_activation_logs_license_id ON activation_logs(license_id);
CREATE INDEX IF NOT EXISTS idx_activation_logs_machine_id ON activation_logs(machine_id);
CREATE INDEX IF NOT EXISTS idx_activation_logs_created_at ON activation_logs(created_at DESC);

-- =============================================
-- 5. Row Level Security (RLS)
-- =============================================

-- 启用 RLS
ALTER TABLE licenses ENABLE ROW LEVEL SECURITY;
ALTER TABLE activation_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE license_tiers ENABLE ROW LEVEL SECURITY;

-- 许可证表策略 - 允许匿名用户验证和激活许可证
CREATE POLICY "Allow anonymous license verification" ON licenses
    FOR SELECT USING (true);

CREATE POLICY "Allow anonymous license activation" ON licenses
    FOR UPDATE USING (true);

-- 激活日志策略 - 允许匿名用户插入记录
CREATE POLICY "Allow anonymous activation logs" ON activation_logs
    FOR INSERT WITH CHECK (true);

CREATE POLICY "Allow select activation logs" ON activation_logs
    FOR SELECT USING (true);

-- 许可证类型策略 - 允许所有人读取
CREATE POLICY "Allow read license tiers" ON license_tiers
    FOR SELECT USING (true);

-- =============================================
-- 6. 函数: 验证许可证
-- =============================================
CREATE OR REPLACE FUNCTION verify_license(
    p_license_key VARCHAR(29),
    p_machine_id VARCHAR(64),
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_license RECORD;
    v_tier RECORD;
    v_result JSONB;
    v_now TIMESTAMPTZ := NOW();
BEGIN
    -- 查找许可证
    SELECT * INTO v_license
    FROM licenses
    WHERE license_key = p_license_key;

    -- 许可证不存在
    IF NOT FOUND THEN
        -- 记录失败日志
        INSERT INTO activation_logs (license_key, machine_id, action, ip_address, user_agent, success, error_message)
        VALUES (p_license_key, p_machine_id, 'verify', p_ip_address, p_user_agent, false, 'LICENSE_NOT_FOUND');

        RETURN jsonb_build_object(
            'valid', false,
            'error_code', 'LICENSE_NOT_FOUND',
            'error_message', '许可证不存在'
        );
    END IF;

    -- 检查许可证状态
    IF v_license.status = 'revoked' THEN
        INSERT INTO activation_logs (license_id, license_key, machine_id, action, ip_address, user_agent, success, error_message)
        VALUES (v_license.id, p_license_key, p_machine_id, 'verify', p_ip_address, p_user_agent, false, 'LICENSE_REVOKED');

        RETURN jsonb_build_object(
            'valid', false,
            'error_code', 'LICENSE_REVOKED',
            'error_message', '许可证已被撤销'
        );
    END IF;

    -- 检查机器绑定
    IF v_license.machine_id IS NOT NULL AND v_license.machine_id != p_machine_id THEN
        INSERT INTO activation_logs (license_id, license_key, machine_id, action, ip_address, user_agent, success, error_message)
        VALUES (v_license.id, p_license_key, p_machine_id, 'verify', p_ip_address, p_user_agent, false, 'MACHINE_MISMATCH');

        RETURN jsonb_build_object(
            'valid', false,
            'error_code', 'MACHINE_MISMATCH',
            'error_message', '许可证已绑定其他设备'
        );
    END IF;

    -- 检查是否过期
    IF v_license.expires_at IS NOT NULL AND v_license.expires_at < v_now THEN
        -- 更新状态为已过期
        UPDATE licenses SET status = 'expired' WHERE id = v_license.id;

        INSERT INTO activation_logs (license_id, license_key, machine_id, action, ip_address, user_agent, success, error_message)
        VALUES (v_license.id, p_license_key, p_machine_id, 'verify', p_ip_address, p_user_agent, false, 'LICENSE_EXPIRED');

        RETURN jsonb_build_object(
            'valid', false,
            'error_code', 'LICENSE_EXPIRED',
            'error_message', '许可证已过期'
        );
    END IF;

    -- 获取许可证类型信息
    SELECT * INTO v_tier FROM license_tiers WHERE id = v_license.tier;

    -- 更新最后验证时间
    UPDATE licenses SET last_verified_at = v_now WHERE id = v_license.id;

    -- 记录成功验证
    INSERT INTO activation_logs (license_id, license_key, machine_id, action, ip_address, user_agent, success)
    VALUES (v_license.id, p_license_key, p_machine_id, 'verify', p_ip_address, p_user_agent, true);

    -- 返回成功结果
    RETURN jsonb_build_object(
        'valid', true,
        'license_info', jsonb_build_object(
            'tier', v_license.tier,
            'tier_name', v_tier.name,
            'is_pro', v_license.tier > 0,
            'activated', v_license.status = 'active',
            'activated_at', v_license.activated_at,
            'expires_at', v_license.expires_at,
            'days_remaining', CASE
                WHEN v_license.expires_at IS NULL THEN NULL
                ELSE GREATEST(0, EXTRACT(DAY FROM v_license.expires_at - v_now)::INTEGER)
            END,
            'features', v_tier.features
        )
    );
END;
$$;

-- =============================================
-- 7. 函数: 激活许可证 (接收系统信息)
-- =============================================
-- 先删除旧版本的函数（如果存在不同参数签名的版本）
DROP FUNCTION IF EXISTS activate_license(VARCHAR, VARCHAR);
DROP FUNCTION IF EXISTS activate_license(VARCHAR, VARCHAR, INET, TEXT);
DROP FUNCTION IF EXISTS activate_license(VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, INET, TEXT);

CREATE OR REPLACE FUNCTION activate_license(
    p_license_key VARCHAR(29),
    p_machine_id VARCHAR(64),
    p_os_info VARCHAR(100) DEFAULT NULL,
    p_os_version VARCHAR(50) DEFAULT NULL,
    p_hostname VARCHAR(100) DEFAULT NULL,
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_license RECORD;
    v_tier RECORD;
    v_now TIMESTAMPTZ := NOW();
    v_expires_at TIMESTAMPTZ;
BEGIN
    -- 查找许可证
    SELECT * INTO v_license
    FROM licenses
    WHERE license_key = p_license_key
    FOR UPDATE;

    -- 许可证不存在
    IF NOT FOUND THEN
        INSERT INTO activation_logs (license_key, machine_id, os_info, os_version, hostname, action, ip_address, user_agent, success, error_message)
        VALUES (p_license_key, p_machine_id, p_os_info, p_os_version, p_hostname, 'activate', p_ip_address, p_user_agent, false, 'LICENSE_NOT_FOUND');

        RETURN jsonb_build_object(
            'success', false,
            'error_code', 'LICENSE_NOT_FOUND',
            'error_message', '许可证不存在'
        );
    END IF;

    -- 检查许可证状态
    IF v_license.status = 'revoked' THEN
        INSERT INTO activation_logs (license_id, license_key, machine_id, os_info, os_version, hostname, action, ip_address, user_agent, success, error_message)
        VALUES (v_license.id, p_license_key, p_machine_id, p_os_info, p_os_version, p_hostname, 'activate', p_ip_address, p_user_agent, false, 'LICENSE_REVOKED');

        RETURN jsonb_build_object(
            'success', false,
            'error_code', 'LICENSE_REVOKED',
            'error_message', '许可证已被撤销'
        );
    END IF;

    -- 检查许可证是否已过期
    IF v_license.status = 'expired' OR (v_license.expires_at IS NOT NULL AND v_license.expires_at < v_now) THEN
        INSERT INTO activation_logs (license_id, license_key, machine_id, os_info, os_version, hostname, action, ip_address, user_agent, success, error_message)
        VALUES (v_license.id, p_license_key, p_machine_id, p_os_info, p_os_version, p_hostname, 'activate', p_ip_address, p_user_agent, false, 'LICENSE_EXPIRED');

        RETURN jsonb_build_object(
            'success', false,
            'error_code', 'LICENSE_EXPIRED',
            'error_message', '许可证已过期'
        );
    END IF;

    -- 检查许可证是否已被激活（绑定到其他机器）
    IF v_license.machine_id IS NOT NULL THEN
        -- 如果是同一台机器，允许重新激活（刷新验证）
        IF v_license.machine_id = p_machine_id THEN
            -- 同一台机器，更新验证时间即可
            UPDATE licenses SET
                last_verified_at = v_now
            WHERE id = v_license.id;

            -- 获取许可证类型信息
            SELECT * INTO v_tier FROM license_tiers WHERE id = v_license.tier;

            INSERT INTO activation_logs (license_id, license_key, machine_id, os_info, os_version, hostname, action, ip_address, user_agent, success)
            VALUES (v_license.id, p_license_key, p_machine_id, p_os_info, p_os_version, p_hostname, 'verify', p_ip_address, p_user_agent, true);

            RETURN jsonb_build_object(
                'success', true,
                'license_info', jsonb_build_object(
                    'tier', v_license.tier,
                    'tier_name', v_tier.name,
                    'is_pro', v_license.tier > 0,
                    'activated', true,
                    'activated_at', v_license.activated_at,
                    'expires_at', v_license.expires_at,
                    'days_remaining', CASE
                        WHEN v_license.expires_at IS NULL THEN NULL
                        ELSE GREATEST(0, EXTRACT(DAY FROM v_license.expires_at - v_now)::INTEGER)
                    END,
                    'features', v_tier.features
                )
            );
        ELSE
            -- 不同机器，拒绝激活
            INSERT INTO activation_logs (license_id, license_key, machine_id, os_info, os_version, hostname, action, ip_address, user_agent, success, error_message)
            VALUES (v_license.id, p_license_key, p_machine_id, p_os_info, p_os_version, p_hostname, 'activate', p_ip_address, p_user_agent, false, 'ALREADY_ACTIVATED');

            RETURN jsonb_build_object(
                'success', false,
                'error_code', 'ALREADY_ACTIVATED',
                'error_message', '许可证已在其他设备上激活，一个许可证只能绑定一台设备'
            );
        END IF;
    END IF;

    -- 获取许可证类型信息
    SELECT * INTO v_tier FROM license_tiers WHERE id = v_license.tier;

    -- 计算到期时间
    IF v_tier.duration_days > 0 THEN
        v_expires_at := v_now + (v_tier.duration_days || ' days')::INTERVAL;
    ELSE
        v_expires_at := NULL; -- 永久
    END IF;

    -- 更新许可证 (包含系统信息)
    UPDATE licenses SET
        machine_id = p_machine_id,
        os_info = p_os_info,
        os_version = p_os_version,
        hostname = p_hostname,
        status = 'active',
        activated_at = COALESCE(activated_at, v_now),
        expires_at = v_expires_at,
        last_verified_at = v_now,
        activation_count = activation_count + 1
    WHERE id = v_license.id;

    -- 记录激活日志
    INSERT INTO activation_logs (license_id, license_key, machine_id, os_info, os_version, hostname, action, ip_address, user_agent, success)
    VALUES (v_license.id, p_license_key, p_machine_id, p_os_info, p_os_version, p_hostname, 'activate', p_ip_address, p_user_agent, true);

    -- 返回成功结果
    RETURN jsonb_build_object(
        'success', true,
        'license_info', jsonb_build_object(
            'tier', v_license.tier,
            'tier_name', v_tier.name,
            'is_pro', v_license.tier > 0,
            'activated', true,
            'activated_at', v_now,
            'expires_at', v_expires_at,
            'days_remaining', CASE
                WHEN v_expires_at IS NULL THEN NULL
                ELSE EXTRACT(DAY FROM v_expires_at - v_now)::INTEGER
            END,
            'features', v_tier.features
        )
    );
END;
$$;

-- =============================================
-- 8. 函数: 撤销许可证激活
-- =============================================
CREATE OR REPLACE FUNCTION deactivate_license(
    p_license_key VARCHAR(29),
    p_machine_id VARCHAR(64),
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_license RECORD;
BEGIN
    -- 查找许可证
    SELECT * INTO v_license
    FROM licenses
    WHERE license_key = p_license_key AND machine_id = p_machine_id
    FOR UPDATE;

    -- 许可证不存在或机器不匹配
    IF NOT FOUND THEN
        INSERT INTO activation_logs (license_key, machine_id, action, ip_address, user_agent, success, error_message)
        VALUES (p_license_key, p_machine_id, 'deactivate', p_ip_address, p_user_agent, false, 'LICENSE_NOT_FOUND_OR_MISMATCH');

        RETURN jsonb_build_object(
            'success', false,
            'error_code', 'LICENSE_NOT_FOUND_OR_MISMATCH',
            'error_message', '许可证不存在或机器不匹配'
        );
    END IF;

    -- 更新许可证状态 (保留 machine_id 和 activated_at，以便记录历史)
    UPDATE licenses SET
        status = 'unused',
        machine_id = NULL
    WHERE id = v_license.id;

    -- 记录撤销日志
    INSERT INTO activation_logs (license_id, license_key, machine_id, action, ip_address, user_agent, success)
    VALUES (v_license.id, p_license_key, p_machine_id, 'deactivate', p_ip_address, p_user_agent, true);

    RETURN jsonb_build_object(
        'success', true,
        'message', '许可证已成功撤销'
    );
END;
$$;

-- =============================================
-- 9. 函数: 生成许可证密钥
-- =============================================
CREATE OR REPLACE FUNCTION generate_license_key()
RETURNS VARCHAR(29)
LANGUAGE plpgsql
AS $$
DECLARE
    v_chars VARCHAR(32) := 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
    v_key VARCHAR(25) := '';
    v_formatted VARCHAR(29);
    i INTEGER;
BEGIN
    -- 生成25个随机字符
    FOR i IN 1..25 LOOP
        v_key := v_key || substr(v_chars, floor(random() * 32 + 1)::INTEGER, 1);
    END LOOP;

    -- 格式化为 XXXXX-XXXXX-XXXXX-XXXXX-XXXXX
    v_formatted := substr(v_key, 1, 5) || '-' ||
                   substr(v_key, 6, 5) || '-' ||
                   substr(v_key, 11, 5) || '-' ||
                   substr(v_key, 16, 5) || '-' ||
                   substr(v_key, 21, 5);

    RETURN v_formatted;
END;
$$;

-- =============================================
-- 10. 函数: 创建新许可证 (管理员使用)
-- =============================================
CREATE OR REPLACE FUNCTION create_license(
    p_tier INTEGER,
    p_notes TEXT DEFAULT NULL,
    p_max_activations INTEGER DEFAULT 1
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_license_key VARCHAR(29);
    v_license_id UUID;
BEGIN
    -- 生成唯一的许可证密钥
    LOOP
        v_license_key := generate_license_key();
        EXIT WHEN NOT EXISTS (SELECT 1 FROM licenses WHERE license_key = v_license_key);
    END LOOP;

    -- 插入许可证
    INSERT INTO licenses (license_key, tier, notes, max_activations)
    VALUES (v_license_key, p_tier, p_notes, p_max_activations)
    RETURNING id INTO v_license_id;

    RETURN jsonb_build_object(
        'success', true,
        'license_key', v_license_key,
        'license_id', v_license_id,
        'tier', p_tier
    );
END;
$$;

-- =============================================
-- 11. 函数: 批量创建许可证 (管理员使用)
-- =============================================
CREATE OR REPLACE FUNCTION create_licenses_batch(
    p_tier INTEGER,
    p_count INTEGER,
    p_notes TEXT DEFAULT NULL,
    p_max_activations INTEGER DEFAULT 1
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_licenses JSONB := '[]'::JSONB;
    v_result JSONB;
    i INTEGER;
BEGIN
    -- 限制批量创建数量
    IF p_count > 100 THEN
        RETURN jsonb_build_object(
            'success', false,
            'error', '批量创建最多支持100个许可证'
        );
    END IF;

    FOR i IN 1..p_count LOOP
        v_result := create_license(p_tier, p_notes, p_max_activations);
        v_licenses := v_licenses || jsonb_build_array(v_result);
    END LOOP;

    RETURN jsonb_build_object(
        'success', true,
        'count', p_count,
        'licenses', v_licenses
    );
END;
$$;

-- =============================================
-- 12. 视图: 许可证统计
-- =============================================
CREATE OR REPLACE VIEW license_stats AS
SELECT
    tier,
    lt.display_name as tier_name,
    COUNT(*) as total,
    COUNT(*) FILTER (WHERE l.status = 'unused') as unused,
    COUNT(*) FILTER (WHERE l.status = 'active') as active,
    COUNT(*) FILTER (WHERE l.status = 'expired') as expired,
    COUNT(*) FILTER (WHERE l.status = 'revoked') as revoked
FROM licenses l
LEFT JOIN license_tiers lt ON l.tier = lt.id
GROUP BY l.tier, lt.display_name;

-- =============================================
-- 13. 授予函数执行权限
-- =============================================
-- 授予 anon 角色执行验证和激活函数的权限
GRANT EXECUTE ON FUNCTION verify_license(VARCHAR, VARCHAR, INET, TEXT) TO anon;
GRANT EXECUTE ON FUNCTION activate_license(VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, INET, TEXT) TO anon;
GRANT EXECUTE ON FUNCTION deactivate_license(VARCHAR, VARCHAR, INET, TEXT) TO anon;

-- 授予 service_role 角色执行创建函数的权限
GRANT EXECUTE ON FUNCTION create_license(INTEGER, TEXT, INTEGER) TO service_role;
GRANT EXECUTE ON FUNCTION create_licenses_batch(INTEGER, INTEGER, TEXT, INTEGER) TO service_role;
GRANT EXECUTE ON FUNCTION generate_license_key() TO service_role;

-- 授予 authenticated 角色 (如果需要登录用户也能调用)
GRANT EXECUTE ON FUNCTION verify_license(VARCHAR, VARCHAR, INET, TEXT) TO authenticated;
GRANT EXECUTE ON FUNCTION activate_license(VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, INET, TEXT) TO authenticated;
GRANT EXECUTE ON FUNCTION deactivate_license(VARCHAR, VARCHAR, INET, TEXT) TO authenticated;

-- 授予所有角色完整权限 (确保 RPC 调用正常)
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON ALL TABLES IN SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON ALL ROUTINES IN SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO anon, authenticated, service_role;

-- 通知 PostgREST 重新加载 schema
NOTIFY pgrst, 'reload schema';

-- =============================================
-- 14. 安全防护: 请求签名和防重放
-- =============================================

-- 请求签名密钥 (用于验证请求来自合法客户端)
-- 注意: 这个密钥需要与客户端保持一致
CREATE TABLE IF NOT EXISTS security_config (
    key VARCHAR(50) PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 插入默认签名密钥 (生产环境应使用更复杂的密钥)
INSERT INTO security_config (key, value) VALUES
('request_sign_key', 'TRACELESS_SIGN_KEY_2024_SECURE_V1')
ON CONFLICT (key) DO NOTHING;

-- 已使用的 nonce 表 (防重放攻击)
CREATE TABLE IF NOT EXISTS used_nonces (
    nonce VARCHAR(64) PRIMARY KEY,
    machine_id VARCHAR(64) NOT NULL,
    used_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创建索引用于清理过期 nonce
CREATE INDEX IF NOT EXISTS idx_used_nonces_used_at ON used_nonces(used_at);

-- 定期清理过期 nonce (保留24小时)
CREATE OR REPLACE FUNCTION cleanup_expired_nonces()
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    DELETE FROM used_nonces WHERE used_at < NOW() - INTERVAL '24 hours';
END;
$$;

-- 可疑活动记录表
CREATE TABLE IF NOT EXISTS security_events (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,  -- 'REPLAY_ATTACK', 'INVALID_SIGNATURE', 'RATE_LIMIT', 'BRUTE_FORCE'
    machine_id VARCHAR(64),
    license_key VARCHAR(29),
    ip_address INET,
    details JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_security_events_created_at ON security_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_machine_id ON security_events(machine_id);

-- IP 频率限制表
CREATE TABLE IF NOT EXISTS rate_limits (
    ip_address INET PRIMARY KEY,
    request_count INTEGER DEFAULT 1,
    first_request_at TIMESTAMPTZ DEFAULT NOW(),
    last_request_at TIMESTAMPTZ DEFAULT NOW()
);

-- 安全激活函数 (带签名验证)
DROP FUNCTION IF EXISTS secure_activate_license(VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, BIGINT, VARCHAR, VARCHAR, INET, TEXT);

CREATE OR REPLACE FUNCTION secure_activate_license(
    p_license_key VARCHAR(29),
    p_machine_id VARCHAR(64),
    p_os_info VARCHAR(100) DEFAULT NULL,
    p_os_version VARCHAR(50) DEFAULT NULL,
    p_hostname VARCHAR(100) DEFAULT NULL,
    p_timestamp BIGINT DEFAULT NULL,           -- Unix 时间戳 (毫秒)
    p_nonce VARCHAR(64) DEFAULT NULL,          -- 随机数防重放
    p_signature VARCHAR(64) DEFAULT NULL,      -- HMAC-SHA256 签名
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_sign_key TEXT;
    v_expected_sig TEXT;
    v_now_ms BIGINT;
    v_time_diff BIGINT;
    v_rate_limit RECORD;
BEGIN
    -- 1. 检查必需的安全参数
    IF p_timestamp IS NULL OR p_nonce IS NULL OR p_signature IS NULL THEN
        INSERT INTO security_events (event_type, machine_id, license_key, ip_address, details)
        VALUES ('MISSING_SECURITY_PARAMS', p_machine_id, p_license_key, p_ip_address,
                jsonb_build_object('timestamp', p_timestamp, 'nonce', p_nonce, 'has_signature', p_signature IS NOT NULL));

        RETURN jsonb_build_object(
            'success', false,
            'error_code', 'SECURITY_ERROR',
            'error_message', '请求参数不完整'
        );
    END IF;

    -- 2. 时间戳验证 (允许5分钟误差)
    v_now_ms := EXTRACT(EPOCH FROM NOW()) * 1000;
    v_time_diff := ABS(v_now_ms - p_timestamp);

    IF v_time_diff > 300000 THEN  -- 5分钟 = 300000毫秒
        INSERT INTO security_events (event_type, machine_id, license_key, ip_address, details)
        VALUES ('TIMESTAMP_EXPIRED', p_machine_id, p_license_key, p_ip_address,
                jsonb_build_object('client_ts', p_timestamp, 'server_ts', v_now_ms, 'diff_ms', v_time_diff));

        RETURN jsonb_build_object(
            'success', false,
            'error_code', 'REQUEST_EXPIRED',
            'error_message', '请求已过期，请检查系统时间'
        );
    END IF;

    -- 3. Nonce 防重放检查
    IF EXISTS (SELECT 1 FROM used_nonces WHERE nonce = p_nonce) THEN
        INSERT INTO security_events (event_type, machine_id, license_key, ip_address, details)
        VALUES ('REPLAY_ATTACK', p_machine_id, p_license_key, p_ip_address,
                jsonb_build_object('nonce', p_nonce));

        RETURN jsonb_build_object(
            'success', false,
            'error_code', 'REPLAY_DETECTED',
            'error_message', '检测到重放攻击'
        );
    END IF;

    -- 4. 获取签名密钥并验证签名
    SELECT value INTO v_sign_key FROM security_config WHERE key = 'request_sign_key';

    -- 计算期望的签名: HMAC-SHA256(license_key + machine_id + timestamp + nonce, sign_key)
    -- 注意: PostgreSQL 使用 encode(hmac(...), 'hex') 来计算 HMAC
    v_expected_sig := encode(
        hmac(
            p_license_key || p_machine_id || p_timestamp::TEXT || p_nonce,
            v_sign_key,
            'sha256'
        ),
        'hex'
    );

    IF p_signature != v_expected_sig THEN
        INSERT INTO security_events (event_type, machine_id, license_key, ip_address, details)
        VALUES ('INVALID_SIGNATURE', p_machine_id, p_license_key, p_ip_address,
                jsonb_build_object('provided_sig', LEFT(p_signature, 16) || '...'));

        RETURN jsonb_build_object(
            'success', false,
            'error_code', 'INVALID_SIGNATURE',
            'error_message', '请求签名验证失败'
        );
    END IF;

    -- 5. IP 频率限制检查 (每分钟最多10次请求)
    -- 仅当 IP 地址不为空时进行限制
    IF p_ip_address IS NOT NULL THEN
        SELECT * INTO v_rate_limit FROM rate_limits WHERE ip_address = p_ip_address;

        IF FOUND THEN
            IF v_rate_limit.last_request_at > NOW() - INTERVAL '1 minute' THEN
                IF v_rate_limit.request_count >= 10 THEN
                    INSERT INTO security_events (event_type, machine_id, license_key, ip_address, details)
                    VALUES ('RATE_LIMIT', p_machine_id, p_license_key, p_ip_address,
                            jsonb_build_object('count', v_rate_limit.request_count));

                    RETURN jsonb_build_object(
                        'success', false,
                        'error_code', 'RATE_LIMITED',
                        'error_message', '请求过于频繁，请稍后重试'
                    );
                ELSE
                    UPDATE rate_limits SET
                        request_count = request_count + 1,
                        last_request_at = NOW()
                    WHERE ip_address = p_ip_address;
                END IF;
            ELSE
                -- 超过1分钟，重置计数
                UPDATE rate_limits SET
                    request_count = 1,
                    first_request_at = NOW(),
                    last_request_at = NOW()
                WHERE ip_address = p_ip_address;
            END IF;
        ELSE
            INSERT INTO rate_limits (ip_address) VALUES (p_ip_address);
        END IF;
    END IF;

    -- 6. 记录已使用的 nonce
    INSERT INTO used_nonces (nonce, machine_id) VALUES (p_nonce, p_machine_id);

    -- 7. 调用原始激活函数
    RETURN activate_license(p_license_key, p_machine_id, p_os_info, p_os_version, p_hostname, p_ip_address, p_user_agent);
END;
$$;

-- 授权安全函数
GRANT EXECUTE ON FUNCTION secure_activate_license(VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, BIGINT, VARCHAR, VARCHAR, INET, TEXT) TO anon;
GRANT EXECUTE ON FUNCTION secure_activate_license(VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, BIGINT, VARCHAR, VARCHAR, INET, TEXT) TO authenticated;

-- 启用 RLS
ALTER TABLE security_config ENABLE ROW LEVEL SECURITY;
ALTER TABLE used_nonces ENABLE ROW LEVEL SECURITY;
ALTER TABLE security_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE rate_limits ENABLE ROW LEVEL SECURITY;

-- 仅允许 service_role 访问安全配置
CREATE POLICY "Only service role can access security_config" ON security_config
    FOR ALL USING (false);

-- 允许插入 nonce
CREATE POLICY "Allow insert nonces" ON used_nonces
    FOR INSERT WITH CHECK (true);

CREATE POLICY "Allow select nonces" ON used_nonces
    FOR SELECT USING (true);

-- 安全事件仅允许插入和服务角色查看
CREATE POLICY "Allow insert security events" ON security_events
    FOR INSERT WITH CHECK (true);

-- 频率限制
CREATE POLICY "Allow rate limit operations" ON rate_limits
    FOR ALL USING (true);

-- =============================================
-- 完成
-- =============================================
COMMENT ON TABLE licenses IS '许可证主表';
COMMENT ON TABLE activation_logs IS '许可证激活/验证日志';
COMMENT ON TABLE license_tiers IS '许可证类型配置';
COMMENT ON TABLE security_config IS '安全配置';
COMMENT ON TABLE used_nonces IS '已使用的随机数(防重放)';
COMMENT ON TABLE security_events IS '安全事件日志';
COMMENT ON TABLE rate_limits IS 'IP请求频率限制';
COMMENT ON FUNCTION verify_license IS '验证许可证有效性';
COMMENT ON FUNCTION activate_license IS '激活许可证';
COMMENT ON FUNCTION secure_activate_license IS '安全激活许可证(带签名验证)';
COMMENT ON FUNCTION deactivate_license IS '撤销许可证激活';
COMMENT ON FUNCTION create_license IS '创建单个许可证';
COMMENT ON FUNCTION create_licenses_batch IS '批量创建许可证';

-- =============================================
-- 15. 函数: 获取许可证列表 (管理员查看)
-- =============================================
CREATE OR REPLACE FUNCTION get_licenses_list(
    p_status VARCHAR(20) DEFAULT NULL,       -- 筛选状态: unused, active, expired, revoked, NULL=全部
    p_tier INTEGER DEFAULT NULL,              -- 筛选类型: 0-3, NULL=全部
    p_page INTEGER DEFAULT 1,                 -- 页码
    p_page_size INTEGER DEFAULT 50            -- 每页数量
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_offset INTEGER;
    v_total INTEGER;
    v_licenses JSONB;
BEGIN
    -- 计算偏移量
    v_offset := (p_page - 1) * p_page_size;

    -- 获取总数
    SELECT COUNT(*) INTO v_total
    FROM licenses l
    WHERE (p_status IS NULL OR l.status = p_status)
      AND (p_tier IS NULL OR l.tier = p_tier);

    -- 获取许可证列表
    SELECT COALESCE(jsonb_agg(
        jsonb_build_object(
            'id', l.id,
            'license_key', l.license_key,
            'tier', l.tier,
            'tier_name', lt.display_name,
            'status', l.status,
            'machine_id', l.machine_id,
            'os_info', l.os_info,
            'os_version', l.os_version,
            'hostname', l.hostname,
            'created_at', l.created_at,
            'activated_at', l.activated_at,
            'expires_at', l.expires_at,
            'last_verified_at', l.last_verified_at,
            'activation_count', l.activation_count,
            'max_activations', l.max_activations,
            'notes', l.notes,
            'days_remaining', CASE
                WHEN l.expires_at IS NULL THEN NULL
                WHEN l.expires_at < NOW() THEN 0
                ELSE GREATEST(0, EXTRACT(DAY FROM l.expires_at - NOW())::INTEGER)
            END
        ) ORDER BY l.created_at DESC
    ), '[]'::JSONB) INTO v_licenses
    FROM licenses l
    LEFT JOIN license_tiers lt ON l.tier = lt.id
    WHERE (p_status IS NULL OR l.status = p_status)
      AND (p_tier IS NULL OR l.tier = p_tier)
    LIMIT p_page_size
    OFFSET v_offset;

    RETURN jsonb_build_object(
        'success', true,
        'total', v_total,
        'page', p_page,
        'page_size', p_page_size,
        'total_pages', CEIL(v_total::DECIMAL / p_page_size),
        'licenses', v_licenses
    );
END;
$$;

-- 授予许可证列表函数权限
GRANT EXECUTE ON FUNCTION get_licenses_list(VARCHAR, INTEGER, INTEGER, INTEGER) TO service_role;
GRANT EXECUTE ON FUNCTION get_licenses_list(VARCHAR, INTEGER, INTEGER, INTEGER) TO authenticated;

-- =============================================
-- 16. 函数: 获取许可证统计信息
-- =============================================
CREATE OR REPLACE FUNCTION get_license_statistics()
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_total INTEGER;
    v_unused INTEGER;
    v_active INTEGER;
    v_expired INTEGER;
    v_revoked INTEGER;
    v_by_tier JSONB;
BEGIN
    -- 分别计算每个状态的数量
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE status = 'unused'),
        COUNT(*) FILTER (WHERE status = 'active'),
        COUNT(*) FILTER (WHERE status = 'expired'),
        COUNT(*) FILTER (WHERE status = 'revoked')
    INTO v_total, v_unused, v_active, v_expired, v_revoked
    FROM licenses;

    -- 计算按类型分组的统计
    SELECT COALESCE(jsonb_object_agg(
        COALESCE(lt.name, 'unknown'),
        jsonb_build_object(
            'total', tier_stats.total,
            'unused', tier_stats.unused,
            'active', tier_stats.active,
            'expired', tier_stats.expired
        )
    ), '{}'::JSONB) INTO v_by_tier
    FROM (
        SELECT
            tier,
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'unused') as unused,
            COUNT(*) FILTER (WHERE status = 'active') as active,
            COUNT(*) FILTER (WHERE status = 'expired') as expired
        FROM licenses
        GROUP BY tier
    ) tier_stats
    LEFT JOIN license_tiers lt ON tier_stats.tier = lt.id;

    RETURN jsonb_build_object(
        'success', true,
        'statistics', jsonb_build_object(
            'total', COALESCE(v_total, 0),
            'unused', COALESCE(v_unused, 0),
            'active', COALESCE(v_active, 0),
            'expired', COALESCE(v_expired, 0),
            'revoked', COALESCE(v_revoked, 0),
            'by_tier', v_by_tier
        )
    );
END;
$$;

-- 授予统计函数权限
GRANT EXECUTE ON FUNCTION get_license_statistics() TO service_role;
GRANT EXECUTE ON FUNCTION get_license_statistics() TO authenticated;

COMMENT ON FUNCTION get_licenses_list IS '获取许可证列表(支持分页和筛选)';
COMMENT ON FUNCTION get_license_statistics IS '获取许可证统计信息';
