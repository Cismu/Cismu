import "./style.css";

window.__TAURI_ISOLATION_HOOK__ = (payload) => {
  return payload;
};
