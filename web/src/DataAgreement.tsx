export default function DataAgreement({ completed, onAccept, disabled = false }: {
  completed: boolean; onAccept: () => void; disabled?: boolean;
}) {
  return completed ? <p className="agreement-completed" role="status"><span aria-hidden="true">✓</span> Data Protection agreement completed</p>
    : <label className="consent"><input type="checkbox" disabled={disabled} checked={false} onChange={onAccept} />
      I agree to the Experimental Preview and Data Protection terms above. I authorize processing by OpenAI, Jev,
      or OSS models as deemed appropriate by the solution, and retention in protected local records.</label>;
}
