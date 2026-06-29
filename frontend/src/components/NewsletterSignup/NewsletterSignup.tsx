"use client";

import { useState, useId } from "react";

interface NewsletterSignupProps {
  /** Called with the validated email when the form is submitted successfully. */
  onSuccess?: (email: string) => void;
}

interface FormState {
  email: string;
  touched: boolean;
  submitted: boolean;
  success: boolean;
}

function validateEmail(email: string): string | null {
  if (email.trim() === "") {
    return "Email address is required.";
  }
  // Basic RFC-5321 surface check: local@domain.tld
  const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!EMAIL_RE.test(email.trim())) {
    return "Please enter a valid email address.";
  }
  return null;
}

export function NewsletterSignup({ onSuccess }: NewsletterSignupProps) {
  const emailId = useId();
  const errorId = useId();

  const [form, setForm] = useState<FormState>({
    email: "",
    touched: false,
    submitted: false,
    success: false,
  });

  const errorMessage = form.touched ? validateEmail(form.email) : null;
  const isInvalid = errorMessage !== null;

  function handleChange(e: React.ChangeEvent<HTMLInputElement>) {
    setForm((prev) => ({ ...prev, email: e.target.value, touched: true }));
  }

  function handleBlur() {
    setForm((prev) => ({ ...prev, touched: true }));
  }

  function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setForm((prev) => ({ ...prev, touched: true, submitted: true }));

    const error = validateEmail(form.email);
    if (error) {
      return;
    }

    setForm((prev) => ({ ...prev, success: true }));
    onSuccess?.(form.email.trim());
  }

  if (form.success) {
    return (
      <div role="status" aria-live="polite" className="newsletter-success">
        <p>SUBSCRIPTION CONFIRMED. WELCOME TO THE NETWORK.</p>
      </div>
    );
  }

  return (
    <form
      onSubmit={handleSubmit}
      noValidate
      aria-label="Newsletter signup"
      className="newsletter-form"
    >
      <div className="newsletter-field">
        <label htmlFor={emailId} className="terminal-label">
          EMAIL_ADDRESS
        </label>

        <input
          id={emailId}
          type="email"
          name="email"
          value={form.email}
          onChange={handleChange}
          onBlur={handleBlur}
          placeholder="ENTER EMAIL..."
          autoComplete="email"
          aria-required="true"
          aria-invalid={isInvalid ? "true" : "false"}
          aria-describedby={isInvalid ? errorId : undefined}
        />

        {isInvalid && (
          <span
            id={errorId}
            role="alert"
            aria-live="assertive"
            className="newsletter-error"
          >
            {errorMessage}
          </span>
        )}
      </div>

      <button type="submit">SUBSCRIBE</button>
    </form>
  );
}

export default NewsletterSignup;
