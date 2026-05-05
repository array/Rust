export type Billing = "monthly" | "yearly";

export interface Subscription {
  id: number;
  name: string;
  price: number;
  billing: Billing;
  category: string;
  next_billing: string; // YYYY-MM-DD
  memo: string;
  created_at: string;
}

export type SubscriptionInput = Omit<Subscription, "id" | "created_at">;

export const CATEGORIES = [
  "動画配信",
  "音楽",
  "クラウドストレージ",
  "ゲーム",
  "ソフトウェア",
  "ニュース",
  "その他",
] as const;
