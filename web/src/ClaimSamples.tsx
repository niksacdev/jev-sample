import { useRef } from "react";
import samples from "./claimSamples.json";

export const defaultClaimMessage = samples[0]?.questions[0] ?? "";

export default function ClaimSamples({ disabled, onSelect }: {
  disabled: boolean; onSelect: (message: string) => void;
}) {
  const nextQuestion = useRef<Record<string, number>>({});
  return <div className="story-buttons" role="group" aria-label="Sample claim questions">
    {samples.map(category => <button key={category.id} type="button" disabled={disabled}
      title={`Load an example from ${category.label}. Click again for another question.`}
      onClick={() => {
        const index = nextQuestion.current[category.id] ?? 0;
        const question = category.questions[index];
        if (question === undefined) throw new Error(`Missing sample question for ${category.id}.`);
        nextQuestion.current[category.id] = (index + 1) % category.questions.length;
        onSelect(question);
      }}>{category.label}</button>)}
  </div>;
}
