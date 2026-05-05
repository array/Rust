import Database from "@tauri-apps/plugin-sql";
import type { Subscription, SubscriptionInput } from "./types";

let _db: Database | null = null;

async function getDb(): Promise<Database> {
  if (!_db) {
    _db = await Database.load("sqlite:subscriptions.db");
  }
  return _db;
}

export async function fetchAll(): Promise<Subscription[]> {
  const db = await getDb();
  const rows = await db.select<Subscription[]>(
    "SELECT * FROM subscriptions ORDER BY next_billing ASC, name ASC"
  );
  return rows;
}

export async function insertSubscription(input: SubscriptionInput): Promise<void> {
  const db = await getDb();
  await db.execute(
    `INSERT INTO subscriptions (name, price, billing, category, next_billing, memo)
     VALUES ($1, $2, $3, $4, $5, $6)`,
    [input.name, input.price, input.billing, input.category, input.next_billing, input.memo]
  );
}

export async function updateSubscription(id: number, input: SubscriptionInput): Promise<void> {
  const db = await getDb();
  await db.execute(
    `UPDATE subscriptions
     SET name=$1, price=$2, billing=$3, category=$4, next_billing=$5, memo=$6
     WHERE id=$7`,
    [input.name, input.price, input.billing, input.category, input.next_billing, input.memo, id]
  );
}

export async function deleteSubscription(id: number): Promise<void> {
  const db = await getDb();
  await db.execute("DELETE FROM subscriptions WHERE id=$1", [id]);
}
