import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { NewsletterSignup } from "./NewsletterSignup";

describe("NewsletterSignup", () => {
  // ---------------------------------------------------------------------------
  // Rendering
  // ---------------------------------------------------------------------------

  it("renders the email input and subscribe button", () => {
    render(<NewsletterSignup />);

    expect(
      screen.getByRole("textbox", { name: /email_address/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /subscribe/i })
    ).toBeInTheDocument();
  });

  it("does not show an error message before the user interacts with the form", () => {
    render(<NewsletterSignup />);

    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("input has aria-invalid=false on initial render", () => {
    render(<NewsletterSignup />);

    const input = screen.getByRole("textbox", { name: /email_address/i });
    expect(input).toHaveAttribute("aria-invalid", "false");
  });

  // ---------------------------------------------------------------------------
  // Empty email validation
  // ---------------------------------------------------------------------------

  it("shows a required error when submitting with an empty email", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    expect(
      await screen.findByText(/email address is required/i)
    ).toBeInTheDocument();
  });

  it("sets aria-invalid=true on the input when submitted empty", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    const input = screen.getByRole("textbox", { name: /email_address/i });
    await waitFor(() =>
      expect(input).toHaveAttribute("aria-invalid", "true")
    );
  });

  it("associates the error message with the input via aria-describedby", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    const input = screen.getByRole("textbox", { name: /email_address/i });
    const errorEl = await screen.findByRole("alert");

    await waitFor(() => {
      const describedBy = input.getAttribute("aria-describedby");
      expect(describedBy).toBeTruthy();
      expect(errorEl.id).toBe(describedBy);
    });
  });

  // ---------------------------------------------------------------------------
  // Invalid email format validation
  // ---------------------------------------------------------------------------

  it("shows a format error for an email without a domain", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    await user.type(
      screen.getByRole("textbox", { name: /email_address/i }),
      "notanemail"
    );
    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    expect(
      await screen.findByText(/please enter a valid email address/i)
    ).toBeInTheDocument();
  });

  it("shows a format error for an email missing the @ symbol", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    await user.type(
      screen.getByRole("textbox", { name: /email_address/i }),
      "userdomain.com"
    );
    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    expect(
      await screen.findByText(/please enter a valid email address/i)
    ).toBeInTheDocument();
  });

  it("shows a format error for an email with no TLD", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    await user.type(
      screen.getByRole("textbox", { name: /email_address/i }),
      "user@domain"
    );
    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    expect(
      await screen.findByText(/please enter a valid email address/i)
    ).toBeInTheDocument();
  });

  it("sets aria-invalid=true for an invalid email format", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    await user.type(
      screen.getByRole("textbox", { name: /email_address/i }),
      "bad-email"
    );
    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    const input = screen.getByRole("textbox", { name: /email_address/i });
    await waitFor(() =>
      expect(input).toHaveAttribute("aria-invalid", "true")
    );
  });

  // ---------------------------------------------------------------------------
  // Blur / touched behaviour
  // ---------------------------------------------------------------------------

  it("shows a required error when the field is blurred empty", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    const input = screen.getByRole("textbox", { name: /email_address/i });
    await user.click(input);
    await user.tab(); // move focus away, triggering blur

    expect(
      await screen.findByText(/email address is required/i)
    ).toBeInTheDocument();
  });

  it("clears the error when a valid email is typed after an invalid attempt", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    // Trigger error
    await user.click(screen.getByRole("button", { name: /subscribe/i }));
    expect(
      await screen.findByText(/email address is required/i)
    ).toBeInTheDocument();

    // Fix the value
    await user.type(
      screen.getByRole("textbox", { name: /email_address/i }),
      "dev@soroban.dev"
    );

    await waitFor(() =>
      expect(screen.queryByRole("alert")).not.toBeInTheDocument()
    );
  });

  it("sets aria-invalid=false once a valid email is entered", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    // Trigger invalid state first
    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    const input = screen.getByRole("textbox", { name: /email_address/i });
    await waitFor(() => expect(input).toHaveAttribute("aria-invalid", "true"));

    // Now type a valid address
    await user.type(input, "dev@soroban.dev");

    await waitFor(() =>
      expect(input).toHaveAttribute("aria-invalid", "false")
    );
  });

  // ---------------------------------------------------------------------------
  // Successful submission
  // ---------------------------------------------------------------------------

  it("accepts a valid email and calls onSuccess with the trimmed address", async () => {
    const user = userEvent.setup();
    const onSuccess = vi.fn();
    render(<NewsletterSignup onSuccess={onSuccess} />);

    await user.type(
      screen.getByRole("textbox", { name: /email_address/i }),
      "dev@soroban.dev"
    );
    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    await waitFor(() => expect(onSuccess).toHaveBeenCalledWith("dev@soroban.dev"));
  });

  it("renders a success message after a valid submission", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    await user.type(
      screen.getByRole("textbox", { name: /email_address/i }),
      "dev@soroban.dev"
    );
    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    expect(
      await screen.findByRole("status")
    ).toHaveTextContent(/subscription confirmed/i);
  });

  it("does not call onSuccess for an empty email", async () => {
    const user = userEvent.setup();
    const onSuccess = vi.fn();
    render(<NewsletterSignup onSuccess={onSuccess} />);

    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    expect(onSuccess).not.toHaveBeenCalled();
  });

  it("does not call onSuccess for an invalid email format", async () => {
    const user = userEvent.setup();
    const onSuccess = vi.fn();
    render(<NewsletterSignup onSuccess={onSuccess} />);

    await user.type(
      screen.getByRole("textbox", { name: /email_address/i }),
      "notvalid"
    );
    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    expect(onSuccess).not.toHaveBeenCalled();
  });

  // ---------------------------------------------------------------------------
  // Accessibility
  // ---------------------------------------------------------------------------

  it("the email input is marked as required via aria-required", () => {
    render(<NewsletterSignup />);

    const input = screen.getByRole("textbox", { name: /email_address/i });
    expect(input).toHaveAttribute("aria-required", "true");
  });

  it("the form has an accessible label", () => {
    render(<NewsletterSignup />);

    expect(
      screen.getByRole("form", { name: /newsletter signup/i })
    ).toBeInTheDocument();
  });

  it("error message has role=alert for immediate announcement", async () => {
    const user = userEvent.setup();
    render(<NewsletterSignup />);

    await user.click(screen.getByRole("button", { name: /subscribe/i }));

    const alert = await screen.findByRole("alert");
    expect(alert).toBeInTheDocument();
  });
});
