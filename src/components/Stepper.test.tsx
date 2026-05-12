import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Stepper } from "./Stepper";

describe("Stepper", () => {
  it("renders the current value and an optional suffix", () => {
    render(<Stepper value={5} onChange={() => {}} suffix="m" />);
    expect(screen.getByText("5")).toBeInTheDocument();
    expect(screen.getByText("m")).toBeInTheDocument();
  });

  it("increments by the given step when the increase button is clicked", () => {
    const onChange = vi.fn();
    render(<Stepper value={10} onChange={onChange} step={5} />);
    fireEvent.click(screen.getByLabelText("Increase"));
    expect(onChange).toHaveBeenCalledWith(15);
  });

  it("decrements by the given step when the decrease button is clicked", () => {
    const onChange = vi.fn();
    render(<Stepper value={10} onChange={onChange} step={5} />);
    fireEvent.click(screen.getByLabelText("Decrease"));
    expect(onChange).toHaveBeenCalledWith(5);
  });

  it("clamps to max and disables the increase button at the ceiling", () => {
    const onChange = vi.fn();
    render(<Stepper value={23} onChange={onChange} min={0} max={23} />);
    const increase = screen.getByLabelText("Increase");
    expect(increase).toBeDisabled();
    fireEvent.click(increase);
    expect(onChange).not.toHaveBeenCalled();
  });

  it("clamps to min and disables the decrease button at the floor", () => {
    const onChange = vi.fn();
    render(<Stepper value={0} onChange={onChange} min={0} max={23} />);
    const decrease = screen.getByLabelText("Decrease");
    expect(decrease).toBeDisabled();
    fireEvent.click(decrease);
    expect(onChange).not.toHaveBeenCalled();
  });

  it("calls onIncrement/onDecrement instead of the default math when provided", () => {
    const onChange = vi.fn();
    const onIncrement = vi.fn();
    const onDecrement = vi.fn();
    render(
      <Stepper
        value={59}
        onChange={onChange}
        onIncrement={onIncrement}
        onDecrement={onDecrement}
      />,
    );
    fireEvent.click(screen.getByLabelText("Increase"));
    fireEvent.click(screen.getByLabelText("Decrease"));
    expect(onIncrement).toHaveBeenCalledOnce();
    expect(onDecrement).toHaveBeenCalledOnce();
    expect(onChange).not.toHaveBeenCalled();
  });

  it("uses canIncrement/canDecrement to override the default disabled state", () => {
    render(
      <Stepper value={59} onChange={() => {}} max={59} canIncrement={true} canDecrement={false} />,
    );
    expect(screen.getByLabelText("Increase")).not.toBeDisabled();
    expect(screen.getByLabelText("Decrease")).toBeDisabled();
  });
});
