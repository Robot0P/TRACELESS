// Supabase 配置
import { createClient } from '@supabase/supabase-js';

export const SUPABASE_URL = 'https://rmiaqewnnmioucpqyryj.supabase.co';
export const SUPABASE_ANON_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InJtaWFxZXdubm1pb3VjcHF5cnlqIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjY1OTUzODEsImV4cCI6MjA4MjE3MTM4MX0.WQBHyjPv5Mp7t_8KBT6vWgSDixfUq4ruSgtGCXlQBeE';

// 创建 Supabase 客户端 (使用 Anon Key 用于客户端应用)
export const supabase = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
  db: {
    schema: 'public',
  },
});

// 验证配置是否有效
export function isSupabaseConfigured(): boolean {
  // 使用变量避免 TypeScript 字面量类型比较警告
  const url = SUPABASE_URL as string;
  const key = SUPABASE_ANON_KEY as string;
  return (
    url !== 'YOUR_SUPABASE_URL' &&
    key !== 'YOUR_SUPABASE_ANON_KEY' &&
    url.includes('supabase.co')
  );
}
