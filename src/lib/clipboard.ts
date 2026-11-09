/**
 * Copy text with a hidden-textarea fallback for WebViews where the async
 * clipboard API is unavailable or rejects.
 * ponytail: no tauri-plugin-clipboard-manager; add it only if the execCommand
 * fallback stops working on a target platform.
 */
export async function copyText(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.setAttribute("readonly", "");
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    ta.remove();
  }
}
