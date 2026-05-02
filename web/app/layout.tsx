import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Panchang — Vedic Almanac",
  description:
    "A modern Vedic almanac: tithi, nakshatra, yoga, karana, and hora with precise transition times for any place on earth."
};

/* Pre-hydration theme init.
   Reads localStorage, falls back to prefers-color-scheme, applies data-theme on
   <html> before React paints to avoid the dark-mode flash. */
const themeInit = `
(function () {
  try {
    var stored = localStorage.getItem('theme');
    var sys = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    document.documentElement.dataset.theme = stored || sys;
  } catch (e) {
    document.documentElement.dataset.theme = 'light';
  }
})();
`.trim();

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeInit }} />
      </head>
      <body>{children}</body>
    </html>
  );
}
