# Contributing

## AI governance

Install ai-rulez 5.3.0 and run `task governance:generate` after editing `.ai-rulez/`.
Run `task governance:check` to validate native OKF sources, the lock, and generated output.

Installing governance hooks requires Python 3.11 or newer.
Use `poly hooks update` and `poly hooks install` to provision the pinned native validation hook.
