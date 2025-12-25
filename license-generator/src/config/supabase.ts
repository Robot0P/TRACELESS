// Supabase Configuration for License Generator
import { createClient } from '@supabase/supabase-js'

export const SUPABASE_URL = 'https://rmiaqewnnmioucpqyryj.supabase.co';
export const SUPABASE_ANON_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InJtaWFxZXdubm1pb3VjcHF5cnlqIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjY1OTUzODEsImV4cCI6MjA4MjE3MTM4MX0.WQBHyjPv5Mp7t_8KBT6vWgSDixfUq4ruSgtGCXlQBeE';

// 管理员密钥 - 用于创建许可证（需要更高权限）
export const SUPABASE_SERVICE_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InJtaWFxZXdubm1pb3VjcHF5cnlqIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc2NjU5NTM4MSwiZXhwIjoyMDgyMTcxMzgxfQ.m6c8JnWdc6kjJ9qUKQ9aK7TWXChlJgm4rXWd_ZCivMk';

// 创建 Supabase 客户端 (使用 Service Role Key 以获得完整权限)
export const supabase = createClient(SUPABASE_URL, SUPABASE_SERVICE_KEY, {
  db: {
    schema: 'public',
  },
});

// Check if Supabase is configured
export function isSupabaseConfigured(): boolean {
  // 使用变量避免 TypeScript 字面量类型比较警告
  const url = SUPABASE_URL as string;
  return (
    url !== 'YOUR_SUPABASE_URL' &&
    url.startsWith('https://')
  );
}

// Check if Service Key is configured (for admin operations)
export function isServiceKeyConfigured(): boolean {
  // 使用变量避免 TypeScript 字面量类型比较警告
  const key = SUPABASE_SERVICE_KEY as string;
  return (
    key !== 'YOUR_SUPABASE_SERVICE_ROLE_KEY' &&
    key.length > 0
  );
}
