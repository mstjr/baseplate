# Why I make this project
For many reasons actually, a lot of CRM arleady exists, but they are either paid software, or open source but not well maintained, or not open source at all or even it is open source but the it is missing some features that I really want to have in a CRM, so I decided to make my own CRM, and I want to make it open source and free for everyone to use and contribute to it.

# Who is this project for
This projet is has the idea to be used by anyone who wants to use a CRM. So small businesses, freelancers, and even individuals who want to manage their contacts and tasks in a more efficient way.

# What is gonna make it valuable
The main value of this project is that it is open source and free for everyone to use and contribute to it. It is also designed to be really customizable, so users can tailor it to their specific needs. You can create your own custom fields, custom views, custom automations and custom data models. It is also designed to be really easy to use, so even non-technical users can use it without any problems.

# The core ideas
Everything in the CRM is a definition with instances. Even the default definitions like contacts, companies, deals, etc. are just definitions with instances. However, few definitions - like contacts, companies, deals, etc. - are marked as system definitions, which means they cannot be deleted, but they can be customized. The same with some fields for those same system definitions, some fields are marked as system fields, which means they cannot be deleted, but they can be customized.

# Default definitions and fields
- Contacts : First name, last name, email, phone number
- Companies : Name, industry, website, Contacts (linked to contacts definition)
- Deals : Name, amount, stage, company (linked to companies definition), contact (linked to contacts definition)
- Tasks : Name, due date, status, related to (linked to any definition)

# User stories
- The user can create a custom data model, with custom fields and custom views.
- The user can create custom automations based on triggers and actions.
- The user can write custom code to extend the functionality of the CRM.
- The user can manage data from definition : create, read, update and delete records.
- The user can manage the definitions and fields : create, read, update and delete definitions and fields.
- The user need to login to access the CRM, and can manage their account settings.
- The user can invite other users to collaborate on the CRM, and manage their permissions.
- The user can create dashboards and reports to visualize their data in a more efficient way.
- The user can integrate the CRM with other tools and services they use, like email marketing tools, calendar, etc.
- The user can create other organizations and manage their data and users in those organizations.
- The user can manage multiple organizations from the same account, and switch between them easily.

# Tech stack
- Backend : Rust with axum framework, and sqlx for database interactions.
- Frontend : Svelte with sveltekit, and tailwindcss for styling.
- Database : Postgresql
- Cache : Redis
- Message broker : RabbitMQ
- Containerization : Docker