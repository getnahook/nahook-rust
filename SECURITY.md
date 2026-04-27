# Security Policy

## Reporting a vulnerability

If you've found a security issue in **nahook-rust**, **do not file a public GitHub issue**.

### Preferred: GitHub Security Advisories

Use the **"Report a vulnerability"** button on the repository's [Security](https://github.com/getnahook/nahook-rust/security) tab. This creates a private advisory only visible to maintainers.

### Backup: email

Email **support@nahook.com** with subject `Security: nahook-rust`. We'll move the conversation off-list as soon as we've acknowledged.

## What to expect

- **Acknowledgment within 48 hours.**
- **Status update within 7 days** (assessment + planned fix timeline).
- **Coordinated disclosure** — typically within 30 days, sooner for severe issues.

## In scope

- Webhook signature verification bypass or weakness in the SDK
- API key / management token leakage in logs, errors, or transports
- Remote code execution in SDK code
- Dependency vulnerabilities affecting installed users (we'll triage and bump as needed)

## Out of scope

- Rate-limiting on the public Nahook API (report via support@nahook.com without the `Security:` prefix)
- Denial-of-service testing against any nahook.com infrastructure
- Social engineering of Nahook team members
- Issues in `nahook.com`, `docs.nahook.com`, the dashboard, or other non-SDK infrastructure (route those to support@nahook.com)

## No bounty program

We don't currently run a paid bug-bounty program. We do credit reporters in the security advisory and (with your permission) on a future security acknowledgments page.

## Versions covered

The latest minor release of nahook-rust is supported. Earlier versions receive fixes only for issues rated High or Critical.
