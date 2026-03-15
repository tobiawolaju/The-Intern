import "dotenv/config";

const GEMINI_API_KEY = process.env.GEMINI_API_KEY || "";
const GEMINI_MODEL = process.env.GEMINI_MODEL || "gemini-flash-latest";

if (!GEMINI_API_KEY) {
  console.error("Missing GEMINI_API_KEY");
  process.exit(1);
}

const body = {
  contents: [{ role: "user", parts: [{ text: "Say hello in one short sentence." }] }],
  generationConfig: { temperature: 0.2, maxOutputTokens: 64 }
};

const url = `https://generativelanguage.googleapis.com/v1beta/models/${GEMINI_MODEL}:generateContent?key=${GEMINI_API_KEY}`;

try {
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body)
  });

  console.log("Status:", res.status);
  const text = await res.text();
  console.log("Raw response:\n", text);
} catch (err) {
  console.error("Request failed", err);
  process.exit(1);
}
