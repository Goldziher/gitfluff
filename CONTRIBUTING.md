# Contributing

## AI governance

Run `mise run governance:generate` after editing `.ai-rulez/`; it runs the latest ai-rulez through npx.
Run `mise run governance:check` to validate native OKF sources, the lock, and generated output.

Installing governance hooks requires Node 22 or newer, which `mise install` provides.
Run `mise run governance:hooks:install` to provision the pinned native validation hook. Existing machine preferences are preserved.
