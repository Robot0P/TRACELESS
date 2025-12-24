import { invoke } from '@tauri-apps/api/core';
import { Modal } from 'antd';
import i18n from '../i18n/config';

/**
 * 权限管理工具
 */
class PermissionManager {
  private static instance: PermissionManager;
  private isAdmin: boolean | null = null;

  private constructor() {}

  public static getInstance(): PermissionManager {
    if (!PermissionManager.instance) {
      PermissionManager.instance = new PermissionManager();
    }
    return PermissionManager.instance;
  }

  /**
   * 检查是否有管理员权限
   */
  async checkAdminPermission(): Promise<boolean> {
    try {
      this.isAdmin = await invoke<boolean>('check_admin_permission');
      return this.isAdmin;
    } catch {
      return false;
    }
  }

  /**
   * 获取当前权限状态
   */
  getAdminStatus(): boolean | null {
    return this.isAdmin;
  }

  /**
   * 检查操作是否需要管理员权限
   */
  async requiresAdmin(operation: string): Promise<boolean> {
    try {
      return await invoke<boolean>('requires_admin', { operation });
    } catch {
      return false;
    }
  }

  /**
   * 获取权限提升指南
   */
  async getElevationGuide(): Promise<string> {
    try {
      return await invoke<string>('get_elevation_guide');
    } catch {
      return 'Please run this application as administrator';
    }
  }

  /**
   * 请求管理员权限
   */
  async requestElevation(): Promise<void> {
    try {
      await invoke('request_admin_elevation');
    } catch (error) {
      throw new Error(String(error));
    }
  }

  /**
   * 显示权限不足对话框
   */
  async showPermissionDialog(_operation: string, operationName: string): Promise<boolean> {
    const guide = await this.getElevationGuide();
    const t = (key: string) => i18n.t(key);

    return new Promise((resolve) => {
      Modal.warning({
        title: t('permission.requiresAdmin'),
        content: (
          <div>
            <p className="mb-3">{t('permission.operationRequiresAdmin').replace('{operationName}', operationName)}</p>
            <p className="text-sm text-slate-400 mb-3">{guide}</p>
            <p className="text-xs text-slate-500">{t('permission.restartForPermission')}</p>
          </div>
        ),
        okText: t('permission.restartAndElevate'),
        cancelText: t('permission.cancel'),
        centered: true,
        onOk: async () => {
          try {
            await this.requestElevation();
            resolve(true);
          } catch (error) {
            Modal.error({
              title: t('permission.elevationFailed'),
              content: String(error),
              centered: true,
            });
            resolve(false);
          }
        },
        onCancel: () => {
          resolve(false);
        },
      });
    });
  }

  /**
   * 检查并请求权限(如果需要)
   */
  async checkAndRequestPermission(
    operation: string,
    operationName: string,
    force: boolean = false
  ): Promise<boolean> {
    // 如果已经是管理员,直接返回true
    if (this.isAdmin === null) {
      await this.checkAdminPermission();
    }

    if (this.isAdmin && !force) {
      return true;
    }

    // 检查操作是否需要权限
    const needsAdmin = await this.requiresAdmin(operation);

    if (!needsAdmin) {
      return true;
    }

    if (this.isAdmin) {
      return true;
    }

    // 显示权限对话框
    return await this.showPermissionDialog(operation, operationName);
  }
}

export default PermissionManager.getInstance();
