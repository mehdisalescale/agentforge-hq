# Your First Workflow

A complete walkthrough: create a company, hire personas, set goals, and run your first agent.

## 1. Create a Company

Navigate to `/companies` and create:

- **Name**: My AI Team
- **Mission**: Build and ship features faster with AI
- **Budget**: $100

## 2. Hire Personas

Go to `/personas`. Browse 100+ specialists across 11 divisions:

- Engineering, Security, Testing, DevOps
- Product, Design, Marketing, Data
- Support, Legal, Executive

Click **Hire** on a persona to add them to your company. This creates:

- An **agent** (with the persona's system prompt and skills)
- An **org position** (linking the agent to your company hierarchy)

Recommended starter team:

- **Senior Software Engineer** — code writing
- **Code Reviewer** — quality checks
- **Technical Writer** — documentation

## 3. Set Goals

Go to `/goals` and create goals for your company:

- "Ship v1.0 by end of month" (status: in_progress)
- "100% test coverage on critical paths" (status: planned)

Active goals are automatically injected into agent context during runs.

## 4. Run an Agent

Go to the **Dashboard** (home page):

1. Select an agent from the dropdown
2. Type a prompt: "Review the auth module for security vulnerabilities"
3. Click **Run**
4. Watch streaming output in real time

After the run:

- **Session** recorded in `/sessions` with full output
- **Cost** tracked and deducted from company budget
- **Analytics** updated at `/analytics`
- **Security scan** runs on all code output

## 5. Review

- `/sessions` — view past runs, export as JSON/Markdown
- `/analytics` — cost trends, per-agent breakdown, success rates
- `/approvals` — governance decisions pending review
