import {
  fetchAll,
  insertSubscription,
  updateSubscription,
  deleteSubscription,
} from "./db";
import "./styles.css";
import type { Subscription, SubscriptionInput, Billing } from "./types";
import { CATEGORIES } from "./types";

// ─── State ───────────────────────────────────────────────────────────────────
let subscriptions: Subscription[] = [];
let editingId: number | null = null;
let filterCategory = "all";

// ─── DOM helpers ─────────────────────────────────────────────────────────────
const $ = <T extends Element>(sel: string, root: ParentNode = document) =>
  root.querySelector<T>(sel)!;

// ─── Monthly equivalent ───────────────────────────────────────────────────────
function toMonthly(price: number, billing: Billing): number {
  return billing === "yearly" ? price / 12 : price;
}

// ─── Days until next billing ──────────────────────────────────────────────────
function daysUntil(dateStr: string): number | null {
  if (!dateStr) return null;
  const diff = new Date(dateStr).getTime() - Date.now();
  return Math.ceil(diff / 86400000);
}

// ─── Category pill class ──────────────────────────────────────────────────────
function categoryClass(cat: string): string {
  const map: Record<string, string> = {
    動画配信: "cat-video",
    音楽: "cat-music",
    クラウドストレージ: "cat-cloud",
    ゲーム: "cat-game",
    ソフトウェア: "cat-sw",
    ニュース: "cat-news",
    その他: "cat-other",
  };
  return map[cat] || "cat-other";
}

// ─── Render ───────────────────────────────────────────────────────────────────
function render(): void {
  const filtered =
    filterCategory === "all"
      ? subscriptions
      : subscriptions.filter((s) => s.category === filterCategory);

  // Summary
  const totalMonthly = subscriptions.reduce(
    (sum, s) => sum + toMonthly(s.price, s.billing),
    0,
  );
  const totalYearly = totalMonthly * 12;
  $("#summary-monthly").textContent =
    `¥${Math.round(totalMonthly).toLocaleString()}`;
  $("#summary-yearly").textContent =
    `¥${Math.round(totalYearly).toLocaleString()}`;
  $("#summary-count").textContent = `${subscriptions.length} 件`;

  // Cards
  const grid = $("#cards-grid");
  if (filtered.length === 0) {
    grid.innerHTML = `
      <div class="empty-state">
        <div class="empty-icon">📋</div>
        <p>サブスクリプションがありません</p>
        <p class="empty-sub">右上の「追加」ボタンから登録してください</p>
      </div>`;
    return;
  }

  grid.innerHTML = filtered
    .map((s) => {
      const monthly = toMonthly(s.price, s.billing);
      const days = daysUntil(s.next_billing);
      const urgency =
        days !== null && days <= 7
          ? "urgent"
          : days !== null && days <= 14
            ? "soon"
            : "";
      const daysLabel =
        days === null
          ? ""
          : days < 0
            ? `<span class="days overdue">期限超過 ${Math.abs(days)}日</span>`
            : days === 0
              ? `<span class="days urgent-label">本日更新</span>`
              : `<span class="days ${urgency}">${days}日後に更新</span>`;

      return `
      <div class="card ${urgency}" data-id="${s.id}">
        <div class="card-header">
          <span class="cat-pill ${categoryClass(s.category)}">${s.category || "未分類"}</span>
          <div class="card-actions">
            <button class="btn-icon edit-btn" data-id="${s.id}" title="編集">✏️</button>
            <button class="btn-icon delete-btn" data-id="${s.id}" title="削除">🗑️</button>
          </div>
        </div>
        <div class="card-name">${escHtml(s.name)}</div>
        <div class="card-price">
          <span class="price-main">¥${s.price.toLocaleString()}</span>
          <span class="price-unit">/${s.billing === "monthly" ? "月" : "年"}</span>
          ${
            s.billing === "yearly"
              ? `<span class="price-monthly">≈ ¥${Math.round(monthly).toLocaleString()}/月</span>`
              : ""
          }
        </div>
        <div class="card-footer">
          ${
            s.next_billing
              ? `<div class="next-billing">
                <span class="billing-label">次回請求</span>
                <span class="billing-date">${s.next_billing}</span>
                ${daysLabel}
               </div>`
              : ""
          }
          ${s.memo ? `<div class="card-memo" title="${escHtml(s.memo)}">💬 ${escHtml(s.memo)}</div>` : ""}
        </div>
      </div>`;
    })
    .join("");

  // Event delegation
  grid.querySelectorAll(".edit-btn").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      const id = Number((e.currentTarget as HTMLElement).dataset.id);
      openModal(id);
    });
  });
  grid.querySelectorAll(".delete-btn").forEach((btn) => {
    btn.addEventListener("click", async (e) => {
      const id = Number((e.currentTarget as HTMLElement).dataset.id);
      if (confirm("このサブスクリプションを削除しますか？")) {
        await deleteSubscription(id);
        await reload();
      }
    });
  });
}

function escHtml(str: string): string {
  return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// ─── Filter bar ───────────────────────────────────────────────────────────────
function renderFilterBar(): void {
  const bar = $("#filter-bar");
  const cats = ["all", ...CATEGORIES];
  bar.innerHTML = cats
    .map(
      (c) =>
        `<button class="filter-btn ${filterCategory === c ? "active" : ""}" data-cat="${c}">
          ${c === "all" ? "すべて" : c}
        </button>`,
    )
    .join("");
  bar.querySelectorAll(".filter-btn").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      filterCategory = (e.currentTarget as HTMLElement).dataset.cat!;
      renderFilterBar();
      render();
    });
  });
}

// ─── Modal ────────────────────────────────────────────────────────────────────
function openModal(id?: number): void {
  editingId = id ?? null;
  const modal = $("#modal");
  const form = $<HTMLFormElement>("#sub-form");
  form.reset();

  if (id !== null && id !== undefined) {
    const s = subscriptions.find((x) => x.id === id)!;
    $<HTMLInputElement>("#f-name").value = s.name;
    $<HTMLInputElement>("#f-price").value = String(s.price);
    $<HTMLSelectElement>("#f-billing").value = s.billing;
    $<HTMLSelectElement>("#f-category").value = s.category;
    $<HTMLInputElement>("#f-next-billing").value = s.next_billing;
    $<HTMLTextAreaElement>("#f-memo").value = s.memo;
    $("#modal-title").textContent = "サブスクリプションを編集";
  } else {
    $("#modal-title").textContent = "サブスクリプションを追加";
    // Default next_billing to today
    const today = new Date().toISOString().split("T")[0];
    $<HTMLInputElement>("#f-next-billing").value = today;
  }

  modal.classList.add("open");
}

function closeModal(): void {
  $("#modal").classList.remove("open");
  editingId = null;
}

// ─── Reload ───────────────────────────────────────────────────────────────────
async function reload(): Promise<void> {
  subscriptions = await fetchAll();
  render();
}

// ─── Init ─────────────────────────────────────────────────────────────────────
async function init(): Promise<void> {
  // Populate category select
  const catSelect = $<HTMLSelectElement>("#f-category");
  CATEGORIES.forEach((c) => {
    const opt = document.createElement("option");
    opt.value = c;
    opt.textContent = c;
    catSelect.appendChild(opt);
  });

  // Filter bar
  renderFilterBar();

  // Buttons
  $("#btn-add").addEventListener("click", () => openModal());
  $("#btn-close-modal").addEventListener("click", closeModal);
  $("#modal").addEventListener("click", (e) => {
    if ((e.target as HTMLElement).id === "modal") closeModal();
  });

  // Form submit
  $<HTMLFormElement>("#sub-form").addEventListener("submit", async (e) => {
    e.preventDefault();
    const input: SubscriptionInput = {
      name: $<HTMLInputElement>("#f-name").value.trim(),
      price: Number($<HTMLInputElement>("#f-price").value),
      billing: $<HTMLSelectElement>("#f-billing").value as Billing,
      category: $<HTMLSelectElement>("#f-category").value,
      next_billing: $<HTMLInputElement>("#f-next-billing").value,
      memo: $<HTMLTextAreaElement>("#f-memo").value.trim(),
    };
    if (editingId !== null) {
      await updateSubscription(editingId, input);
    } else {
      await insertSubscription(input);
    }
    closeModal();
    await reload();
  });

  await reload();
}

document.addEventListener("DOMContentLoaded", init);
