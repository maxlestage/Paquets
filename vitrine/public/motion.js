// Orbite — les mouvements de la page : apparitions au défilement, compteurs, cartes
// inclinées, boutons magnétiques, curseur, menu, filtres, installation de la PWA.
// Les scènes 3D, elles, sont en Rust (WebAssembly) : voir src/scenes.rs.
(() => {
  "use strict";
  const doc = document.documentElement;
  const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const fine = matchMedia("(pointer: fine)").matches;
  const $ = (s, el = document) => Array.from(el.querySelectorAll(s));

  // Apparitions et compteurs.
  const compter = (el) => {
    const fin = Number(el.dataset.count) || 0;
    if (reduce) { el.textContent = fin; return; }
    const t0 = performance.now(), duree = 1800;
    const pas = (t) => {
      const x = Math.min(1, (t - t0) / duree);
      el.textContent = Math.round(fin * (1 - Math.pow(1 - x, 4)));
      if (x < 1) requestAnimationFrame(pas);
    };
    requestAnimationFrame(pas);
  };
  const observer = "IntersectionObserver" in window
    ? new IntersectionObserver((entries) => {
        for (const e of entries) {
          if (!e.isIntersecting) continue;
          e.target.classList.add("is-in");
          $("[data-count]", e.target).forEach(compter);
          if (e.target.matches("[data-count]")) compter(e.target);
          observer.unobserve(e.target);
        }
      }, { rootMargin: "0px 0px -8% 0px", threshold: 0.12 })
    : null;
  const observerTout = () => {
    $("[data-reveal]:not(.is-in), .split:not(.is-in), .step:not(.is-in)").forEach((el) => {
      if (observer) observer.observe(el); else el.classList.add("is-in");
    });
  };
  observerTout();
  // Les îlots s'hydratent après : on observe aussi ce qu'ils ajoutent.
  new MutationObserver(observerTout).observe(document.body, { childList: true, subtree: true });

  // Barre de progression, en-tête, frise, grand nom du pied de page.
  const header = document.querySelector(".site-header");
  const progress = document.querySelector(".progress");
  const ligne = document.querySelector(".timeline-line");
  const timeline = document.querySelector(".timeline");
  const mot = document.querySelector(".footer-word");
  let dernier = 0, enAttente = false;
  const defiler = () => {
    enAttente = false;
    const y = scrollY, h = doc.scrollHeight - innerHeight;
    if (progress) progress.style.setProperty("--p", h > 0 ? (y / h).toFixed(4) : 0);
    if (header) {
      header.classList.toggle("scrolled", y > 20);
      const menu = doc.classList.contains("menu-open");
      header.classList.toggle("hide", !menu && y > 400 && y > dernier + 4);
      if (y < dernier - 4) header.classList.remove("hide");
    }
    if (ligne && timeline) {
      const r = timeline.getBoundingClientRect();
      const t = (innerHeight * 0.6 - r.top) / r.height;
      ligne.style.setProperty("--t", Math.max(0, Math.min(1, t)).toFixed(3));
    }
    if (mot) {
      const r = mot.getBoundingClientRect();
      const f = 1 - (r.top - innerHeight * 0.55) / (innerHeight * 0.45);
      mot.style.setProperty("--fw", Math.max(0, Math.min(1, f)).toFixed(3));
    }
    dernier = y;
  };
  addEventListener("scroll", () => { if (!enAttente) { enAttente = true; requestAnimationFrame(defiler); } }, { passive: true });
  addEventListener("resize", defiler, { passive: true });
  defiler();

  // Menu mobile.
  const burger = document.querySelector(".burger");
  const fermer = () => { doc.classList.remove("menu-open"); burger && burger.setAttribute("aria-expanded", "false"); };
  if (burger) {
    burger.addEventListener("click", () => {
      const ouvert = doc.classList.toggle("menu-open");
      burger.setAttribute("aria-expanded", String(ouvert));
      burger.setAttribute("aria-label", ouvert ? "Fermer le menu" : "Ouvrir le menu");
    });
    $(".nav a").forEach((a) => a.addEventListener("click", fermer));
    addEventListener("keydown", (e) => { if (e.key === "Escape") fermer(); });
  }

  // Revenir en haut.
  $(".to-top").forEach((b) => b.addEventListener("click", () => scrollTo({ top: 0, behavior: reduce ? "auto" : "smooth" })));

  // Filtres des réalisations.
  const filtres = $("[data-filter]");
  filtres.forEach((b) => b.addEventListener("click", () => {
    const f = b.dataset.filter;
    filtres.forEach((x) => x.setAttribute("aria-pressed", String(x === b)));
    $(".project[data-cat]").forEach((p, i) => {
      const montre = f === "*" || p.dataset.cat === f;
      if (montre) {
        p.classList.remove("hidden", "is-in");
        p.style.setProperty("--d", `${(i % 3) * 90}ms`);
        requestAnimationFrame(() => requestAnimationFrame(() => p.classList.add("is-in")));
      } else {
        p.classList.add("hidden");
      }
    });
  }));

  // Installation de la PWA.
  let invite = null;
  addEventListener("beforeinstallprompt", (e) => {
    e.preventDefault();
    invite = e;
    $(".install-app").forEach((b) => (b.hidden = false));
  });
  $(".install-app").forEach((b) => b.addEventListener("click", async () => {
    if (!invite) return;
    invite.prompt();
    await invite.userChoice.catch(() => null);
    invite = null;
    b.hidden = true;
  }));
  addEventListener("appinstalled", () => $(".install-app").forEach((b) => (b.hidden = true)));

  if (reduce || !fine) return;

  // Cartes inclinées en 3D, avec une lueur sous le pointeur.
  const lier = (el) => {
    if (el.dataset.bound) return;
    el.dataset.bound = "1";
    el.addEventListener("pointermove", (e) => {
      const r = el.getBoundingClientRect();
      const x = (e.clientX - r.left) / r.width, y = (e.clientY - r.top) / r.height;
      el.style.setProperty("--mx", `${x * 100}%`);
      el.style.setProperty("--my", `${y * 100}%`);
      if (el.classList.contains("tilt")) {
        el.classList.add("tilting");
        el.style.setProperty("--rx", `${(0.5 - y) * 10}deg`);
        el.style.setProperty("--ry", `${(x - 0.5) * 12}deg`);
      }
    });
    el.addEventListener("pointerleave", () => {
      el.classList.remove("tilting");
      el.style.setProperty("--rx", "0deg");
      el.style.setProperty("--ry", "0deg");
    });
  };

  // Boutons magnétiques.
  const aimanter = (el) => {
    if (el.dataset.mag) return;
    el.dataset.mag = "1";
    el.addEventListener("pointermove", (e) => {
      const r = el.getBoundingClientRect();
      const x = e.clientX - r.left - r.width / 2, y = e.clientY - r.top - r.height / 2;
      el.style.transform = `translate(${x * 0.25}px, ${y * 0.3}px)`;
    });
    el.addEventListener("pointerleave", () => { el.style.transform = ""; });
  };
  const brancher = () => { $(".tilt, .glow").forEach(lier); $(".magnetic").forEach(aimanter); };
  brancher();
  new MutationObserver(brancher).observe(document.body, { childList: true, subtree: true });

  // Le curseur : un point et un anneau qui le suit en douceur.
  const curseur = document.querySelector(".cursor");
  if (curseur) {
    const point = curseur.querySelector(".cursor-dot"), anneau = curseur.querySelector(".cursor-ring");
    let mx = -100, my = -100, ax = -100, ay = -100;
    addEventListener("pointermove", (e) => {
      mx = e.clientX; my = e.clientY;
      curseur.classList.add("on");
      point.style.transform = `translate3d(${mx}px, ${my}px, 0)`;
      const cible = e.target.closest && e.target.closest("a, button, summary, label, input, select, textarea, canvas");
      curseur.classList.toggle("hover", !!cible);
    }, { passive: true });
    addEventListener("pointerdown", () => curseur.classList.add("down"));
    addEventListener("pointerup", () => curseur.classList.remove("down"));
    document.addEventListener("pointerleave", () => curseur.classList.remove("on"));
    const suivre = () => {
      ax += (mx - ax) * 0.18; ay += (my - ay) * 0.18;
      anneau.style.transform = `translate3d(${ax}px, ${ay}px, 0)`;
      requestAnimationFrame(suivre);
    };
    requestAnimationFrame(suivre);
  }

  // Le cube CSS du héros suit le pointeur.
  const cube = document.querySelector(".hero-cube");
  if (cube) {
    addEventListener("pointermove", (e) => {
      cube.style.translate = `${(e.clientX / innerWidth - 0.5) * -40}px ${(e.clientY / innerHeight - 0.5) * -30}px`;
    }, { passive: true });
  }
})();
