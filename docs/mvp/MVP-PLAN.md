# AI Security RelayNode — MVP Plan

> **Project:** ai-security-relaynode
> **Created:** 2026-03-24
> **Status:** Planning
> **Goal:** A cross-platform desktop app that turns any computer into a Nostr Relay + IPFS Node with security monitoring

---

## Persona Archetypes

Six archetypes across two factions. Each drives different user stories and surfaces different MVP requirements.

---

### Faction 1: Nomad Super Culture

> Homesteaders, van dwellers, sailing communities, digital nomads, intentional communities — people who need sovereign infrastructure that travels with them or runs off-grid.

#### Persona A: "River" — The Mobile Node Runner

- **Who:** Lives in a converted van/bus. Moves between BLM land, national forests, and nomad gatherings. Has a Raspberry Pi or mini-PC running 24/7 off solar.
- **Technical skill:** Can follow a README. Has used Linux. Not a developer.
- **Connectivity:** Starlink, campground WiFi, phone tethering. Bandwidth is precious and intermittent.
- **Motivation:** Wants to participate in the decentralized web without depending on anyone's server. Believes in digital sovereignty.
- **Frustration:** Every self-hosted tool requires SSH, Docker, CLI, or a VPS. Just wants to click "Start" and have it work.

**River's User Stories:**
| # | Story | MVP? |
|---|-------|------|
| A1 | As River, I want to **install the app in one step** (download → run) so I don't need to compile anything or install dependencies. | **YES** |
| A2 | As River, I want to **click one button to start my relay and IPFS node** so I can provide infrastructure to my community without CLI knowledge. | **YES** |
| A3 | As River, I want to **see my relay's status** (connected peers, events relayed, storage used) so I know it's actually working. | **YES** |
| A4 | As River, I want the app to **use less than 200MB RAM** so it runs on my Raspberry Pi alongside other services. | **YES** |
| A5 | As River, I want my relay to **automatically resume after a reboot** so connectivity gaps from driving/power loss don't require manual intervention. | Phase 2 |
| A6 | As River, I want to **limit storage usage** (e.g., 5GB max for IPFS) so it doesn't fill my small SSD. | Phase 2 |
| A7 | As River, I want to **restrict my relay to known pubkeys** so only my community uses my limited bandwidth. | Phase 2 |

---

#### Persona B: "Sage" — The Gathering Organizer

- **Who:** Coordinates nomad meetups, rainbow gatherings, and homesteader skill-shares. Sets up temporary mesh infrastructure for events.
- **Technical skill:** Intermediate. Comfortable with networking basics. Runs a few servers.
- **Connectivity:** Event sites often have no internet. Uses local mesh or a single shared uplink.
- **Motivation:** Wants a local-first communication layer for gatherings that doesn't require internet. Nostr events can sync when connectivity returns.
- **Frustration:** Existing tools assume always-on internet. Need something that works purely local, then syncs.

**Sage's User Stories:**
| # | Story | MVP? |
|---|-------|------|
| B1 | As Sage, I want to **run the relay on a local network** so people at a gathering can exchange Nostr messages without internet. | **YES** |
| B2 | As Sage, I want to **see how many clients are connected** to my relay so I can troubleshoot during events. | **YES** |
| B3 | As Sage, I want **IPFS to store content locally** so files shared at a gathering persist even without internet. | **YES** |
| B4 | As Sage, I want to **enable basic spam protection** (rate limiting) so one misbehaving client can't flood the relay. | **YES** |
| B5 | As Sage, I want to **sync events with other relays** when internet becomes available so our gathering's messages reach the wider Nostr network. | Phase 2 |
| B6 | As Sage, I want to **export a relay invite link/QR code** so attendees can easily point their Nostr clients at my relay. | Phase 2 |

---

#### Persona C: "Flint" — The Homestead Sysadmin

- **Who:** Runs a permanent off-grid homestead or intentional community. Has stable solar power, a server closet, and a satellite uplink.
- **Technical skill:** Advanced. Can SSH, configure firewalls, read logs.
- **Connectivity:** Stable but low-bandwidth satellite or fixed wireless.
- **Motivation:** Wants to be a reliable node in the decentralized network. Believes in providing infrastructure as community service.
- **Frustration:** Wants fine-grained control over what the relay accepts and stores, but current options require editing config files and restarting services.

**Flint's User Stories:**
| # | Story | MVP? |
|---|-------|------|
| C1 | As Flint, I want to **configure which NIP event kinds my relay accepts** so I can specialize (e.g., text notes only, no media). | Phase 2 |
| C2 | As Flint, I want to **view a security log** of blocked/flagged events so I can audit what the relay rejected and why. | **YES** |
| C3 | As Flint, I want to **set bandwidth limits** so my satellite uplink isn't saturated by relay traffic. | Phase 2 |
| C4 | As Flint, I want the **status dashboard to show uptime, events/hour, and storage growth** so I can monitor long-term health. | **YES** |
| C5 | As Flint, I want to **run the app headless** (no GUI, just the backend) so I can run it on a server without a display. | Phase 2 |

---

### Faction 2: Earth Intelligence Network (Starcom)

> Coordinated teams running security operations, threat intelligence, and forensic investigations across a decentralized network. Trust is explicit, compartmentalized, and auditable.

#### Persona D: "Onyx" — The Field Operator

- **Who:** Deploys to locations with a laptop. Needs secure comms with the team. Operates in environments where centralized services may be monitored or blocked.
- **Technical skill:** Trained on specific tools. Not a developer. Follows SOPs.
- **Connectivity:** Variable — sometimes good, sometimes hostile/unreliable networks.
- **Motivation:** Needs to send and receive messages that can't be intercepted, blocked, or attributed to a centralized service.
- **Frustration:** Currently uses Signal/Telegram — centralized, phone-number-linked, and trivially blocked by network operators.

**Onyx's User Stories:**
| # | Story | MVP? |
|---|-------|------|
| D1 | As Onyx, I want to **connect to my team's relay** and exchange encrypted Nostr messages so comms work even if mainstream services are blocked. | **YES** |
| D2 | As Onyx, I want the **relay to validate event signatures** so I know messages actually came from verified team members. | **YES** |
| D3 | As Onyx, I want to **store evidence files on IPFS** (photos, documents) so they're content-addressed and tamper-evident. | **YES** |
| D4 | As Onyx, I want the app to **work without configuration** — point at a relay address and go. | **YES** |
| D5 | As Onyx, I want **my relay to reject events from unknown pubkeys** so only authorized team members can post. | Phase 2 |
| D6 | As Onyx, I want to **run my own relay if cut off from the team's** so I have local infrastructure until reconnection. | **YES** (same as A2) |

---

#### Persona E: "Cipher" — The Network Overseer

- **Who:** Monitors multiple relay nodes across the network. Responsible for detecting anomalies, compromised nodes, or adversarial activity.
- **Technical skill:** High. Security background. Reads logs, writes rules.
- **Connectivity:** Stable. Operates from a secure facility or hardened endpoint.
- **Motivation:** Needs visibility across the network. Wants to catch bad actors before they spread.
- **Frustration:** No unified view of relay health and security events across nodes. Currently would have to SSH into each one.

**Cipher's User Stories:**
| # | Story | MVP? |
|---|-------|------|
| E1 | As Cipher, I want to **see security alerts** (rate limit violations, invalid signatures, oversized events) in the dashboard. | **YES** |
| E2 | As Cipher, I want **configurable security rules** (max event size, rate per pubkey, banned event kinds) so I can tune the bot without code changes. | **YES** |
| E3 | As Cipher, I want to **block a pubkey** from the dashboard when I identify a bad actor. | Phase 2 |
| E4 | As Cipher, I want to **export security logs** for forensic analysis. | Phase 2 |
| E5 | As Cipher, I want to **see relay metrics over time** (events/min, unique pubkeys, rejection rate) to detect anomaly patterns. | Phase 3 |
| E6 | As Cipher, I want to **push security policies to multiple nodes** from one dashboard. | Phase 3 |

---

#### Persona F: "Vex" — The Intelligence Analyst

- **Who:** Investigates security incidents. Needs to track evidence, timelines, and findings. Coordinates with other analysts.
- **Technical skill:** Moderate. Uses web apps daily. Not a sysadmin.
- **Connectivity:** Stable.
- **Motivation:** Needs an investigation workflow that ties into the relay's data — events, content, identity — rather than a disconnected ticketing system.
- **Frustration:** Generic investigation tools (Jira, TheHive) don't understand Nostr events or IPFS content hashes.

**Vex's User Stories:**
| # | Story | MVP? |
|---|-------|------|
| F1 | As Vex, I want to **create an investigation** and attach Nostr events and IPFS content as evidence. | Phase 2 |
| F2 | As Vex, I want to **search relay events by pubkey, time range, or content** to find relevant evidence. | Phase 2 |
| F3 | As Vex, I want **evidence to be immutable** (content-addressed) so chain of custody is provable. | Phase 2 |
| F4 | As Vex, I want to **share investigation findings** with team members via the relay. | Phase 3 |

---

## MVP Feature Matrix (derived from user stories)

Mapping YES stories to concrete features:

| Feature | Personas | Stories |
|---------|----------|---------|
| **One-click install** (Tauri native installers) | River, Onyx | A1, D4 |
| **Start/Stop relay + IPFS from GUI** | River, Sage, Onyx | A2, D6 |
| **Working Nostr relay** (NIP-01: EVENT, REQ, CLOSE) | All | A2, B1, D1 |
| **Event signature validation** | Onyx, Cipher | D2 |
| **Working IPFS storage** (store + retrieve by hash) | Sage, Onyx | B3, D3 |
| **Status dashboard** (connections, events, storage) | River, Sage, Flint | A3, B2, C4 |
| **Security alert feed** (violations, rejections) | Flint, Cipher | C2, E1 |
| **Rate limiting** (per-connection, configurable) | Sage, Cipher | B4, E2 |
| **Configurable security rules** (max size, rate, etc.) | Cipher | E2 |
| **Low resource usage** (<200MB RAM) | River | A4 |

---

## MVP Scope Summary

**The app does three things on launch day:**

1. **Relay** — Accept Nostr WebSocket connections, validate events, store in SQLite, serve subscriptions. NIP-01 compliant.
2. **Node** — Store and retrieve content-addressed data. Accessible via the relay and API.
3. **Monitor** — Apply configurable security rules to relay traffic. Log violations. Show alerts in the GUI.

**Wrapped in:** A Tauri desktop app with a status dashboard, Start/Stop controls, and a security alert feed. Installable via .deb/.AppImage (Linux), .msi (Windows), .dmg (Mac).

**What's explicitly NOT in MVP:**
- Investigation CRUD (Phase 2 — Vex's stories)
- Bridge/subnet coordination (Phase 3 — multi-node)
- AI/ML security analysis (Phase 3+)
- Pubkey allowlist/blocklist management (Phase 2)
- Headless mode (Phase 2)
- Event sync between relays (Phase 2)
- Clearance levels / Earth Alliance profiles (Phase 3)
