# OpenCRM

A modern, developer-friendly, and **truly customizable** open-source CRM built for speed and flexibility.

## Why This Project?

While the market is flooded with CRM solutions, most fall into two traps: they are either expensive proprietary silos or open-source projects that lack critical flexibility (like subforms or granular field requirements).

After experimenting with excellent tools like [Twenty](https://twenty.com), I realized there was still a gap for a CRM that treats **customization as a first-class citizen**. This project is my answer: a free, open-source alternative that gives you total control over your data structures without the bloat.

## Key Value Propositions

* **Definition-Based Architecture:** Everything is a "Definition" with "Instances." This allows for infinite nesting and custom data modeling.
* **High Performance:** Built with **Rust** for a lightning-fast, memory-safe backend.
* **Total Customization:** Create custom fields, views, automations, and even write custom code to extend functionality.
* **Multi-Tenant by Design:** Manage multiple organizations from a single account effortlessly.
* **User-Centric:** Designed to be intuitive for non-technical users while remaining powerful for developers.

---

## Core Architecture & Data Model

The system operates on a "System vs. Custom" logic. While you can build anything, we provide a robust foundation:

### Default Definitions

| Entity        | Key System Fields                                |
| ------------- | ------------------------------------------------ |
| **Contacts**  | First name, Last name, Email, Phone              |
| **Companies** | Name, Industry, Website, Linked Contacts         |
| **Deals**     | Name, Amount, Stage, Linked Company/Contact      |
| **Tasks**     | Name, Due Date, Status, Polymorphic "Related To" |

> **Note:** System definitions and fields cannot be deleted to ensure core stability, but they are fully customizable to fit your workflow.

---

## Tech Stack

We've chosen a modern, high-performance stack to ensure the CRM can scale with your business:

* **Backend:** [Rust](https://www.rust-lang.org/) (Axum framework + SQLx)
* **Frontend:** [SvelteKit](https://kit.svelte.dev/) + [TailwindCSS](https://tailwindcss.com/)
* **Database:** PostgreSQL
* **Caching:** Redis
* **Messaging:** RabbitMQ (for background jobs and automations)
* **Infrastructure:** Docker

---

## Roadmap & User Stories

Currently, the project is in **Early Alpha**. Our development is guided by these core capabilities:

* [ ] **Data Modeling:** Create/Read/Update/Delete (CRUD) for custom definitions and fields.
* [x] **Automation Engine:** Trigger-based actions and custom code execution.
* [ ] **Collaboration:** Robust user permission system and organization switching.
* [ ] **Insights:** Custom dashboards and reporting engines.
* [ ] **Integrations:** API-first approach for syncing with Email, Calendars, and Marketing tools.

---

## Alternatives

If you need a production-ready CRM today, I highly recommend checking out these projects:

* **Twenty:** Modern, sleek, and community-driven.
* **SuiteCRM:** The enterprise-grade open-source classic.
* **EspoCRM:** Highly flexible and lean.

---

## Contributing

This is an open-source project for everyone. Whether you want to fix a bug, suggest a feature, or help with documentation, your contributions are welcome!

*Stay tuned for contribution guidelines as we move toward a stable release.*