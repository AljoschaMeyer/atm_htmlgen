const {
  computePosition,
  flip,
  shift,
  offset,
  inline,
  autoUpdate,
} = window.FloatingUIDOM;

let body = document.querySelector("body");

let root_preview = null;

function leaf_preview() {
  function lf(current) {
    if (current.child) {
      return lf(current.child);
    } else {
      return current;
    }
  }

  if (root_preview) {
    return lf(root_preview);
  } else {
    return null;
  }
}

function clean_stack() {
  let preview = leaf_preview();

  while (preview != null && !preview.hovered) {
    if (preview.node) {
      preview.node.remove();
      preview.ref.removeEventListener("mouseenter", preview.ref_onmouseenter);
      preview.ref.removeEventListener("mouseleave", preview.ref_onmouseleave);
    }

    if (preview.parent) {
      preview.parent.child = null;
      preview = preview.parent;
    } else {
      root_preview = null;
      return;
    }
  }
}

function wait(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function schedule_preview_removal(preview) {
  wait(300)
  .then(() => {
    clean_stack();
  });
}

function position_preview({clientX, clientY, ref: target, node: tooltip}) {
  computePosition(target, tooltip, {
    placement: 'top',
    middleware: [
      offset(6),
      flip(),
      inline({x: clientX, y: clientY}),
      shift({padding: 5}),
    ],
  }).then(({x, y}) => {
    Object.assign(tooltip.style, {
      left: `${x}px`,
      top: `${y}px`,
    });
  });
}

function find_preview_url(elem) {
  if (elem.dataset && elem.dataset.preview) {
    return elem.dataset.preview;
  }

  if (elem.parentNode) {
    return find_preview_url(elem.parentNode);
  }

  return null;
}

body.addEventListener("mouseover", (evt) => {
  const preview_url = find_preview_url(evt.target);

  if (preview_url) {
    let do_not_display = false;
    const cancel_listener = () => {
      do_not_display = true;
    };
    evt.target.addEventListener("mouseleave", cancel_listener);

    Promise.all([
      wait(400),
      fetch(preview_url)
        .then(response => {
          if (!response.ok) {
            return "Could not load preview. You can still click the reference to jump to its target.";
          } else {
            return response.text();
          }
        }),
    ]).then(([_, content]) => {
      if (!do_not_display) {
        evt.target.removeEventListener("mouseleave", cancel_listener);

        clean_stack();

        const preview = {
          clientX: evt.clientX,
          clientY: evt.clientY,
          ref: evt.target,
          hovered: true,
        };

        const parent = leaf_preview();
        if (parent) {
          parent.child = preview;
          preview.parent = parent;
        } else {
          root_preview = preview;
          preview.parent = null;
        }

        preview.ref_onmouseenter = preview.ref.addEventListener("mouseenter", () => {
          preview.hovered = true;
        });
        preview.ref_onmouseleave = preview.ref.addEventListener("mouseleave", () => {
          preview.hovered = false;
          schedule_preview_removal(preview);
        });

        const content_node = document.createElement("div");
        content_node.classList.add("preview_content");
        content_node.innerHTML = content;

        const preview_node = document.createElement("div");
        preview_node.classList.add("preview");
        preview_node.addEventListener('mouseenter', () => {
          preview.hovered = true;
        });
        preview_node.addEventListener("mouseleave", () => {
          preview.hovered = false;
          schedule_preview_removal(preview);
        });
        preview_node.appendChild(content_node);

        preview.node = preview_node;
        position_preview(preview);

        body.appendChild(preview_node);
      }
    });
  }
});
