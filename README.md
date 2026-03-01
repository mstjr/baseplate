# Baseplate

A modern, developer-friendly, and **truly customizable** open-source CRM built for speed and flexibility.

<div style="background-color: #fffbeb; border-left: 4px solid #f59e0b; padding: 1.5rem; margin-bottom: 2rem; border-radius: 0.375rem; color: #92400e;">
  <div style="display: flex; align-items: center; margin-bottom: 0.5rem;">
    <svg style="width: 1.5rem; height: 1.5rem; margin-right: 0.75rem;" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path>
    </svg>
    <h2 style="margin: 0; font-size: 1.25rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em;">Status: WIP</h2>
  </div>
  <p style="margin: 0; line-height: 1.6;">
    <strong>Baseplate is under active construction.</strong> The architecture and core concepts are evolving rapidly. 
    Expect <strong>breaking changes</strong> and database schema shifts without migration paths between early versions. 
    <br><br>
    <em>Proceed with caution: This is not currently suitable for production use.</em>
    <br><br>
    <em>Some features presented in this README may not be implemented yet or may change significantly.</em>
  </p>
</div>

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

### Default Definitions (not done yet)

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

Currently, the project is in **WIP**. Our development is guided by these core capabilities:

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