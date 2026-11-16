(function () {
  const tabsContainer = document.getElementById("tabs-container");
  const btnBrand = document.getElementById("btn-brand");
  const btnNewTab = document.getElementById("btn-new-tab");
  const btnBack = document.getElementById("btn-back");
  const btnForward = document.getElementById("btn-forward");
  const btnReload = document.getElementById("btn-reload");
  const btnHome = document.getElementById("btn-home");
  const btnReopen = document.getElementById("btn-reopen");
  const btnBookmark = document.getElementById("btn-bookmark");
  const btnZoom = document.getElementById("btn-zoom");
  const btnFind = document.getElementById("btn-find");
  const btnXray = document.getElementById("btn-xray");
  const btnDevtools = document.getElementById("btn-devtools");
  const iconReload = document.getElementById("icon-reload");
  const iconStop = document.getElementById("icon-stop");
  const omniboxBadge = document.getElementById("omnibox-badge");
  const omniboxForm = document.getElementById("omnibox-form");
  const omnibox = document.getElementById("omnibox");

  let currentState = {
    tabs: [],
    active_tab_id: null,
    active_url: "",
    is_loading: false,
    is_bookmarked: false,
    zoom_percent: 100,
    can_reopen_closed: false,
    bookmarks: [],
  };

  let lastSyncedTabId = null;
  let lastSyncedUrl = "";
  let omniboxEditing = false;

  function post(payload) {
    if (window.ipc && typeof window.ipc.postMessage === "function") {
      window.ipc.postMessage(JSON.stringify(payload));
    }
  }

  function createGlobeSvg() {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("width", "14");
    svg.setAttribute("height", "14");
    svg.setAttribute("viewBox", "0 0 16 16");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "currentColor");
    svg.setAttribute("stroke-width", "1.4");
    svg.innerHTML =
      '<circle cx="8" cy="8" r="5.5"/><path d="M2.5 8h11M8 2.5a8.5 8.5 0 0 1 0 11M8 2.5a8.5 8.5 0 0 0 0 11"/>';
    return svg;
  }

  function createCloseSvg() {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("width", "11");
    svg.setAttribute("height", "11");
    svg.setAttribute("viewBox", "0 0 12 12");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "currentColor");
    svg.setAttribute("stroke-width", "1.6");
    svg.setAttribute("stroke-linecap", "round");
    svg.innerHTML = '<path d="M2.5 2.5l7 7M9.5 2.5l-7 7"/>';
    return svg;
  }

  function updateTabIcon(iconWrap, tab) {
    const targetMode = tab.is_loading
      ? "loading"
      : tab.favicon
        ? "img:" + tab.favicon
        : "globe";

    if (iconWrap.dataset.iconMode === targetMode) {
      return;
    }

    iconWrap.dataset.iconMode = targetMode;
    iconWrap.innerHTML = "";

    if (tab.is_loading) {
      const spinner = document.createElement("span");
      spinner.className = "spinner";
      iconWrap.appendChild(spinner);
    } else if (tab.favicon) {
      const img = document.createElement("img");
      img.src = tab.favicon;
      img.alt = "";
      img.onerror = function () {
        iconWrap.dataset.iconMode = "globe";
        iconWrap.innerHTML = "";
        iconWrap.appendChild(createGlobeSvg());
      };
      iconWrap.appendChild(img);
    } else {
      iconWrap.appendChild(createGlobeSvg());
    }
  }

  function createTabElement(tab) {
    const el = document.createElement("div");
    el.dataset.tabId = String(tab.id);
    el.setAttribute("role", "tab");

    const iconWrap = document.createElement("span");
    iconWrap.className = "tab-icon";

    const titleEl = document.createElement("span");
    titleEl.className = "tab-title";

    const closeBtn = document.createElement("button");
    closeBtn.type = "button";
    closeBtn.className = "tab-close";
    closeBtn.title = "Close Tab (Ctrl+W)";
    closeBtn.setAttribute("aria-label", "Close tab");
    closeBtn.appendChild(createCloseSvg());

    closeBtn.addEventListener("click", function (e) {
      e.stopPropagation();
      post({ cmd: "close_tab", id: Number(el.dataset.tabId) });
    });

    el.addEventListener("click", function () {
      if (!el.classList.contains("active")) {
        post({ cmd: "switch_tab", id: Number(el.dataset.tabId) });
      }
    });

    el.addEventListener("dblclick", function () {
      post({ cmd: "pin_tab", id: Number(el.dataset.tabId) });
    });

    el.addEventListener("contextmenu", function (e) {
      e.preventDefault();
      post({ cmd: "pin_tab", id: Number(el.dataset.tabId) });
    });

    el.addEventListener("auxclick", function (e) {
      if (e.button === 1) {
        e.preventDefault();
        post({ cmd: "close_tab", id: Number(el.dataset.tabId) });
      }
    });

    el.appendChild(iconWrap);
    el.appendChild(titleEl);
    el.appendChild(closeBtn);
    return el;
  }

  function renderTabs(tabs) {
    const existingById = new Map();
    for (const child of Array.from(tabsContainer.children)) {
      if (child.dataset && child.dataset.tabId) {
        existingById.set(child.dataset.tabId, child);
      }
    }

    const activeIds = new Set();
    let activeEl = null;

    tabs.forEach(function (tab, index) {
      const idKey = String(tab.id);
      activeIds.add(idKey);

      let el = existingById.get(idKey);
      if (!el) {
        el = createTabElement(tab);
      }

      const nextClass =
        "tab" +
        (tab.active ? " active" : "") +
        (tab.pinned ? " pinned" : "");
      if (el.className !== nextClass) {
        el.className = nextClass;
      }

      const selectedVal = tab.active ? "true" : "false";
      if (el.getAttribute("aria-selected") !== selectedVal) {
        el.setAttribute("aria-selected", selectedVal);
      }

      const tabTitle = tab.title || "New Tab";
      const nextTooltip = tabTitle + " (Double-click to pin)";
      if (el.title !== nextTooltip) {
        el.title = nextTooltip;
      }

      const iconWrap = el.children[0];
      const titleEl = el.children[1];
      if (titleEl.textContent !== tabTitle) {
        titleEl.textContent = tabTitle;
      }
      updateTabIcon(iconWrap, tab);

      const currentAtIdx = tabsContainer.children[index];
      if (currentAtIdx !== el) {
        tabsContainer.insertBefore(el, currentAtIdx || null);
      }

      if (tab.active) {
        activeEl = el;
      }
    });

    for (const [idKey, el] of existingById.entries()) {
      if (!activeIds.has(idKey)) {
        el.remove();
      }
    }

    if (activeEl) {
      activeEl.scrollIntoView({ block: "nearest", inline: "nearest" });
    }
  }

  function updateToolbar(state) {
    if (state.is_loading) {
      iconReload.classList.add("hidden");
      iconStop.classList.remove("hidden");
      btnReload.title = "Stop loading";
      btnReload.setAttribute("aria-label", "Stop loading");
    } else {
      iconReload.classList.remove("hidden");
      iconStop.classList.add("hidden");
      btnReload.title = "Reload (Ctrl+R)";
      btnReload.setAttribute("aria-label", "Reload");
    }

    btnReopen.disabled = !state.can_reopen_closed;

    const tabSwitched = state.active_tab_id !== lastSyncedTabId;
    const urlChanged = state.active_url !== lastSyncedUrl;

    if (!omniboxEditing || tabSwitched) {
      omnibox.value = state.active_url || "";
      lastSyncedTabId = state.active_tab_id;
      lastSyncedUrl = state.active_url || "";
    } else if (urlChanged && document.activeElement !== omnibox) {
      omnibox.value = state.active_url || "";
      lastSyncedUrl = state.active_url || "";
    }

    const url = state.active_url || "";
    const isInternal =
      url.startsWith("pluto://") || url.startsWith("adit://") || url === "";
    if (url.startsWith("https://")) {
      omniboxBadge.textContent = "HTTPS";
      omniboxBadge.className = "omnibox-badge badge-https";
    } else if (isInternal) {
      omniboxBadge.textContent = "Pluto";
      omniboxBadge.className = "omnibox-badge badge-pluto";
    } else {
      omniboxBadge.textContent = "HTTP";
      omniboxBadge.className = "omnibox-badge badge-search";
    }

    const zoom = state.zoom_percent || 100;
    btnZoom.textContent = zoom + "%";
    btnZoom.classList.toggle("hidden", zoom === 100);

    btnBookmark.classList.toggle("bookmarked", Boolean(state.is_bookmarked));
    btnBookmark.title = state.is_bookmarked
      ? "Remove bookmark"
      : "Bookmark this page";
  }

  window.__onBrowserState = function (state) {
    currentState = state;
    renderTabs(state.tabs || []);
    updateToolbar(state);
  };

  window.__focusOmnibox = function () {
    omnibox.focus();
    omnibox.select();
  };

  btnBrand.addEventListener("click", function () {
    post({ cmd: "open_onboarding" });
  });

  btnNewTab.addEventListener("click", function () {
    post({ cmd: "new_tab", url: null });
  });

  btnBack.addEventListener("click", function () {
    post({ cmd: "back" });
  });

  btnForward.addEventListener("click", function () {
    post({ cmd: "forward" });
  });

  btnReload.addEventListener("click", function () {
    if (currentState.is_loading) {
      post({ cmd: "stop" });
    } else {
      post({ cmd: "reload" });
    }
  });

  btnHome.addEventListener("click", function () {
    post({ cmd: "home" });
  });

  btnReopen.addEventListener("click", function () {
    post({ cmd: "reopen_closed_tab" });
  });

  btnZoom.addEventListener("click", function () {
    post({ cmd: "zoom_reset" });
  });

  btnBookmark.addEventListener("click", function () {
    post({ cmd: "toggle_bookmark" });
  });

  btnFind.addEventListener("click", function () {
    post({ cmd: "toggle_find_bar" });
  });

  btnXray.addEventListener("click", function () {
    post({ cmd: "toggle_inspector" });
  });

  btnDevtools.addEventListener("click", function () {
    post({ cmd: "open_devtools" });
  });

  omnibox.addEventListener("focus", function () {
    omniboxEditing = true;
    setTimeout(function () {
      if (document.activeElement === omnibox) {
        omnibox.select();
      }
    }, 0);
  });

  omnibox.addEventListener("input", function () {
    omniboxEditing = true;
  });

  omnibox.addEventListener("blur", function () {
    omniboxEditing = false;
  });

  omniboxForm.addEventListener("submit", function (e) {
    e.preventDefault();
    const input = omnibox.value;
    omniboxEditing = false;
    omnibox.blur();
    post({ cmd: "navigate", input: input });
  });

  window.addEventListener(
    "keydown",
    function (e) {
      const ctrl = e.ctrlKey || e.metaKey;
      const key = (e.key || "").toLowerCase();

      if (e.key === "F12" || (ctrl && e.shiftKey && key === "i")) {
        e.preventDefault();
        post({ cmd: "open_devtools" });
      } else if (ctrl && e.shiftKey && (key === "d" || key === "c")) {
        e.preventDefault();
        post({ cmd: "toggle_inspector" });
      } else if (ctrl && !e.shiftKey && !e.altKey && key === "f") {
        e.preventDefault();
        post({ cmd: "toggle_find_bar" });
      } else if (ctrl && e.shiftKey && key === "t") {
        e.preventDefault();
        post({ cmd: "reopen_closed_tab" });
      } else if (ctrl && !e.altKey && key === "t") {
        e.preventDefault();
        post({ cmd: "new_tab", url: null });
      } else if (ctrl && !e.altKey && key === "w") {
        e.preventDefault();
        post({ cmd: "close_active_tab" });
      } else if (ctrl && !e.altKey && key === "l") {
        e.preventDefault();
        window.__focusOmnibox();
      } else if ((ctrl && !e.altKey && key === "r") || e.key === "F5") {
        e.preventDefault();
        post({ cmd: "reload" });
      } else if (ctrl && e.key === "Tab") {
        e.preventDefault();
        post({ cmd: e.shiftKey ? "prev_tab" : "next_tab" });
      } else if (e.altKey && !ctrl && e.key === "ArrowLeft") {
        e.preventDefault();
        post({ cmd: "back" });
      } else if (e.altKey && !ctrl && e.key === "ArrowRight") {
        e.preventDefault();
        post({ cmd: "forward" });
      } else if (ctrl && !e.altKey && (key === "=" || key === "+")) {
        e.preventDefault();
        post({ cmd: "zoom_in" });
      } else if (ctrl && !e.altKey && key === "-") {
        e.preventDefault();
        post({ cmd: "zoom_out" });
      } else if (ctrl && !e.altKey && key === "0") {
        e.preventDefault();
        post({ cmd: "zoom_reset" });
      }
    },
    true
  );

  post({ cmd: "ui_ready" });
})();
