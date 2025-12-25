/**
 * 安全工具模块 - 请求签名和加密
 */

// 签名密钥 (需要与服务端保持一致)
// 注意: 生产环境应使用更安全的方式存储密钥
const SIGN_KEY = 'TRACELESS_SIGN_KEY_2024_SECURE_V1';

/**
 * 生成随机 nonce (防重放攻击)
 */
export function generateNonce(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
}

/**
 * 获取当前时间戳 (毫秒)
 */
export function getTimestamp(): number {
  return Date.now();
}

/**
 * 计算 HMAC-SHA256 签名
 * @param message 要签名的消息
 * @param key 签名密钥
 * @returns 十六进制签名字符串
 */
export async function hmacSha256(message: string, key: string): Promise<string> {
  const encoder = new TextEncoder();
  const keyData = encoder.encode(key);
  const messageData = encoder.encode(message);

  const cryptoKey = await crypto.subtle.importKey(
    'raw',
    keyData,
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );

  const signature = await crypto.subtle.sign('HMAC', cryptoKey, messageData);
  const hashArray = Array.from(new Uint8Array(signature));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

/**
 * 生成请求签名
 * @param licenseKey 许可证密钥
 * @param machineId 机器ID
 * @param timestamp 时间戳
 * @param nonce 随机数
 * @returns 签名字符串
 */
export async function generateRequestSignature(
  licenseKey: string,
  machineId: string,
  timestamp: number,
  nonce: string
): Promise<string> {
  const message = licenseKey + machineId + timestamp.toString() + nonce;
  return hmacSha256(message, SIGN_KEY);
}

/**
 * 生成安全请求参数
 * @param licenseKey 许可证密钥
 * @param machineId 机器ID
 * @returns 包含签名的请求参数
 */
export async function generateSecureParams(
  licenseKey: string,
  machineId: string
): Promise<{
  timestamp: number;
  nonce: string;
  signature: string;
}> {
  const timestamp = getTimestamp();
  const nonce = generateNonce();
  const signature = await generateRequestSignature(licenseKey, machineId, timestamp, nonce);

  return {
    timestamp,
    nonce,
    signature,
  };
}

/**
 * 混淆字符串 (简单的字符替换)
 * 用于在内存中保护敏感数据
 */
export function obfuscateString(str: string): string {
  const key = 0x5A;
  return Array.from(str)
    .map(char => String.fromCharCode(char.charCodeAt(0) ^ key))
    .join('');
}

/**
 * 反混淆字符串
 */
export function deobfuscateString(str: string): string {
  // XOR 是对称的，所以反混淆和混淆用同一个函数
  return obfuscateString(str);
}

/**
 * 验证时间戳有效性 (5分钟内)
 */
export function isTimestampValid(timestamp: number): boolean {
  const now = Date.now();
  const diff = Math.abs(now - timestamp);
  return diff <= 5 * 60 * 1000; // 5分钟
}

/**
 * 简单的完整性校验
 * 检查关键代码是否被篡改
 */
export function integrityCheck(): boolean {
  // 检查关键函数是否存在
  if (typeof generateNonce !== 'function') return false;
  if (typeof hmacSha256 !== 'function') return false;
  if (typeof generateRequestSignature !== 'function') return false;

  // 检查 crypto API 是否可用
  if (!crypto || !crypto.subtle) return false;

  return true;
}

/**
 * 检测开发者工具是否打开
 * 通过检测控制台行为来判断
 */
export function detectDevTools(): boolean {
  // 检测 Firebug
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  if ((window as any).Firebug?.chrome?.isInitialized) {
    return true;
  }

  // 检测控制台输出的时间差
  const startTime = performance.now();
  // eslint-disable-next-line no-debugger
  // debugger 语句会在开发者工具打开时暂停
  const endTime = performance.now();
  if (endTime - startTime > 100) {
    return true;
  }

  return false;
}

/**
 * 生成客户端指纹
 * 用于识别客户端环境
 */
export function generateClientFingerprint(): string {
  const components: string[] = [];

  // 屏幕信息
  components.push(`${screen.width}x${screen.height}`);
  components.push(`${screen.colorDepth}`);

  // 时区
  components.push(Intl.DateTimeFormat().resolvedOptions().timeZone);

  // 语言
  components.push(navigator.language);

  // 平台
  components.push(navigator.platform);

  // 硬件并发数
  components.push(String(navigator.hardwareConcurrency || 0));

  // 用户代理
  components.push(navigator.userAgent);

  // 组合并生成哈希
  const fingerprint = components.join('|');

  // 简单哈希
  let hash = 0;
  for (let i = 0; i < fingerprint.length; i++) {
    const char = fingerprint.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32bit integer
  }

  return Math.abs(hash).toString(16);
}

/**
 * 防止控制台调试的保护措施
 * 在生产环境中禁用控制台输出
 */
export function disableConsoleInProduction(): void {
  if (import.meta.env.PROD) {
    const noop = () => {};
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const consoleMethods = ['log', 'debug', 'info', 'warn', 'error', 'table', 'trace', 'dir', 'dirxml', 'group', 'groupEnd', 'time', 'timeEnd', 'profile', 'profileEnd', 'count'] as const;
    consoleMethods.forEach((method) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (console as any)[method] = noop;
    });
  }
}

/**
 * 安全的 JSON 解析
 * 防止 JSON 注入攻击
 */
export function safeJsonParse<T>(json: string, defaultValue: T): T {
  try {
    const parsed = JSON.parse(json);
    // 检查是否包含危险的 __proto__ 或 constructor
    const jsonStr = JSON.stringify(parsed);
    if (jsonStr.includes('__proto__') || jsonStr.includes('constructor')) {
      console.warn('Potentially dangerous JSON detected');
      return defaultValue;
    }
    return parsed as T;
  } catch {
    return defaultValue;
  }
}

/**
 * 生成安全的随机 ID
 */
export function generateSecureId(length: number = 16): string {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
}
