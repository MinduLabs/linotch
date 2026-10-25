/** Tooltip for text cut off by an ellipsis: after a short hover, shows the full text over the element, inside the pill. */
export function tip(node: HTMLElement, text: string) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let box: HTMLElement | null = null;

  const hide = () => {
    clearTimeout(timer);
    box?.remove();
    box = null;
  };
  const show = () => {
    if (!text || node.scrollWidth <= node.clientWidth + 1) return;
    const pill = node.closest(".pill")?.getBoundingClientRect() ?? { left: 0, right: window.innerWidth, top: 0, bottom: window.innerHeight };
    const r = node.getBoundingClientRect();
    box = document.createElement("div");
    box.className = "tip";
    box.textContent = text;
    Object.assign(box.style, {
      position: "fixed", zIndex: "50", pointerEvents: "none", boxSizing: "border-box",
      maxWidth: `${pill.right - pill.left - 28}px`, minWidth: `${Math.min(r.width, pill.right - pill.left - 28)}px`,
      padding: "5px 9px", borderRadius: "10px", background: "#2c2c2e", color: "#fff",
      font: "inherit", fontSize: "12px", fontWeight: "500", lineHeight: "1.3", boxShadow: "0 4px 14px rgba(0,0,0,.6), 0 0 0 1px rgba(255,255,255,.08)",
      left: "0px", top: "0px", visibility: "hidden",
    });
    document.body.append(box);
    const b = box.getBoundingClientRect();
    const left = Math.min(Math.max(r.left - 9, pill.left + 8), pill.right - b.width - 8);
    const top = Math.min(r.top - 5, pill.bottom - b.height - 6);
    Object.assign(box.style, { left: `${left}px`, top: `${top}px`, visibility: "visible" });
  };

  const enter = () => {
    clearTimeout(timer);
    timer = setTimeout(show, 350);
  };
  node.addEventListener("mouseenter", enter);
  node.addEventListener("mouseleave", hide);
  node.addEventListener("click", hide);
  return {
    update(t: string) {
      text = t;
      if (box) box.textContent = t;
    },
    destroy() {
      hide();
      node.removeEventListener("mouseenter", enter);
      node.removeEventListener("mouseleave", hide);
      node.removeEventListener("click", hide);
    },
  };
}
