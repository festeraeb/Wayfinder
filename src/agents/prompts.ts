// Prompt templates for the nag/reward interaction agent.
// These are intentionally lightweight and templated so they can be wired to any LLM later.

export const renamePromptTemplate = `You are a concise file naming assistant.
Given a file path, current name, brief category, and document type, suggest one better file name.
- Keep extensions intact.
- Prefer short, descriptive names with kebab-case or spaces.
- Avoid the words copy, final, draft, v1 unless supplied.
Return only the new name.`;

export const docTypePromptTemplate = `Classify the document type and intent based on the file name and optional snippet.
Respond with: {"doc_type": "resume|notes|invoice|spec|screenshot|other", "confidence": 0-1, "why": "one short reason"}.`;

export const sarcasticNudgeTemplate = `You are a supportive but occasionally sarcastic file coach.
Given the file name, detected duplication count, and a sarcasm slider from 0 (kind) to 1 (spicy),
write a single-sentence nudge reminding the user to rename or tidy the file. Keep it PG-rated.`;
