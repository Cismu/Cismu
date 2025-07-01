export type TauriPayload = Record<string, unknown>;
export type IsolationHook = (payload: TauriPayload) => TauriPayload;

declare global {
  interface Window {
    __TAURI_ISOLATION_HOOK__?: IsolationHook;
  }
}
