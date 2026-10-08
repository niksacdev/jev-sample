import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import ClaimSamples from "./ClaimSamples";
import samples from "./claimSamples.json";

afterEach(cleanup);

test("the three JSON categories cycle independently and never select a missing question", () => {
  const select = vi.fn();
  const view = render(<ClaimSamples disabled={false} onSelect={select} />);
  expect(screen.getAllByRole("button")).toHaveLength(3);
  for (const category of samples) {
    expect(category.questions.length).toBeGreaterThan(1);
    expect(new Set(category.questions).size).toBe(category.questions.length);
    const button = screen.getByRole("button", { name: category.label });
    for (const question of [...category.questions, category.questions[0]]) {
      fireEvent.click(button);
      expect(select).toHaveBeenLastCalledWith(question);
      expect(question?.length).toBeLessThanOrEqual(4000);
    }
  }
  view.rerender(<ClaimSamples disabled onSelect={select} />);
  select.mockClear();
  for (const button of screen.getAllByRole("button")) fireEvent.click(button);
  expect(select).not.toHaveBeenCalled();
});
