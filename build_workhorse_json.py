#!/usr/bin/env python3
"""Generate workhorse-software-engineer.json — interview prep for Workhorse.

Target role: Software Engineer – Automation & Operations (October 2026).
PHP / MySQL / AWS platform; Jira delivery with AI-assisted dev tools; helpdesk
support and the AI-powered RAG pipeline (drafting, resolution, escalation);
SQL, schema changes and migrations; CI/CD; AWS monitoring and alerting;
implementation support and the outsourced infrastructure team.

Uses the Typeset schema 2 layout (opener/callout/facts/steps plus v1 blocks),
as in ~/dev/myers. json.dumps handles all escaping so the output is always valid.
"""
import json

# ---- block helpers ---------------------------------------------------------
def opener(eyebrow, title, intro=None):
    b = {"type": "opener", "eyebrow": eyebrow, "title": title}
    if intro:
        b["intro"] = intro
    return b

def h(level, text):         return {"type": "heading", "level": level, "text": text}
def p(text):                return {"type": "paragraph", "text": text}
def md(content):            return {"type": "markdown", "content": content}
def code(lang, content):    return {"type": "code", "language": lang, "content": content}
def bullets(items):         return {"type": "bullet_list", "items": items}
def table(headers, rows):   return {"type": "table", "headers": headers, "rows": rows}
def facts(pairs):           return {"type": "facts", "items": [{"label": l, "value": v} for l, v in pairs]}
def callout(tone, title, body):
    return {"type": "callout", "tone": tone, "title": title, "body": body}
def why(body):              return callout("brand", "Why it matters", body)
def say(body):              return callout("good", "Say this", body)
def steps(groups):
    return {"type": "steps", "groups": [
        {"title": t, "items": [{"title": a, "detail": b} for a, b in items]} for t, items in groups
    ]}

pages = []
def page(*blocks):
    pages.append({"blocks": list(blocks)})

# ---- contents --------------------------------------------------------------
page({"type": "contents", "title": "Contents"})

# ---- 1. the role -----------------------------------------------------------
page(
    opener("The role", "What they're hiring for",
           "A broad, practical engineering role on a PHP / MySQL / AWS platform: ship Jira tickets "
           "with AI-assisted tools, keep the helpdesk moving with automation, and keep the platform healthy."),
    facts([
        ("Title", "Software Engineer – Automation & Operations"),
        ("Stack", "PHP, MySQL, AWS"),
        ("Day to day", "Jira tickets, scoped, built, reviewed, tested and deployed with their existing AI-assisted workflow"),
        ("The big project", "Their AI-powered RAG pipeline for the helpdesk: better ticket enrichment, then automated drafting, resolution and escalation"),
        ("Also", "SQL and migrations, CI/CD, AWS monitoring and alerting, implementation support, the outsourced infrastructure team"),
        ("Level", "Mid-level. Strong SQL, sound fundamentals, deployment, automation and cloud experience"),
        ("Note", "PHP is *beneficial but not essential*, so your PHP is a clear advantage"),
    ]),
    callout("info", "Read between the lines",
            "**Improving existing systems rather than always building from scratch** and **confident in "
            "assessing the quality and reliability of AI-generated outputs** are the two phrases to land. "
            "They already have the AI tools; they want someone who makes them *reliable*, not someone who "
            "just generates more code."),
)

page(
    opener("The role, mapped", "What they want, what to show",
           "Every responsibility in the job description, with the evidence to bring. Fill the gaps tonight."),
    table(["They want", "What to show"], [
        ["Deliver from Jira tickets", "How you take a vague ticket to a scoped, tested, deployed change."],
        ["AI-assisted development", "How you use Claude Code day to day, and how you review what it writes."],
        ["Helpdesk support and RAG", "An LLM or retrieval feature you built; how you'd measure and improve it."],
        ["Strong SQL", "A query you optimised, a schema you designed, a migration you ran safely."],
        ["CI/CD and automation", "A pipeline you built; a repetitive task you automated away."],
        ["AWS monitoring", "Metrics and alarms you've set up, and an incident you caught or diagnosed."],
        ["Investigation", "A production bug you tracked down: how you found it, how you fixed it."],
        ["Working with others", "Helping non-developers (helpdesk, implementation) and outside specialists."],
    ]),
    callout("good", "Your strongest stories",
            "**Typeset**: a render API you built and run in production, which hit memory limits on a "
            "small server; you diagnosed the OOMs, added swap, and know concurrency and timeouts are next. "
            "That is *monitoring, investigation and operations* in one story. **Claude Code**: you build "
            "with coding agents every day and review their output. **PHP / Laravel**: years of it."),
)

# ---- 2. jira delivery ------------------------------------------------------
page(
    opener("Software development", "From Jira ticket to production",
           "They want ownership: you help scope the ticket, not just pick it up."),
    why("**Take ownership of Jira development tickets, help scope technical requirements and deliver** "
        "is the first responsibility listed. Show a repeatable process."),
    steps([
        ("Scope", [
            ("Understand the problem", "Who reported it, what they expected, how to reproduce it. Ask before you build."),
            ("Find the code", "Trace the path through the codebase; note what else touches it."),
            ("Write acceptance criteria", "What done looks like, edge cases, and what's out of scope. Add it to the ticket."),
        ]),
        ("Build", [
            ("Plan with the agent", "Give the AI tool the ticket, the relevant files and the constraints; agree a plan first."),
            ("Small, focused changes", "One concern per PR; no drive-by refactors in a bug fix."),
            ("Test", "A failing test that reproduces the bug, then the fix. Cover the edge cases from scoping."),
        ]),
        ("Ship", [
            ("Review", "Read every line yourself before anyone else does, then a peer review."),
            ("Deploy", "Through the pipeline, with a way to roll back; database changes go first and stay backward compatible."),
            ("Close the loop", "Check it in production, update the ticket, tell the person who reported it."),
        ]),
    ]),
    say("*\"I treat the ticket as the start of a conversation. I'll reproduce it, write down what done "
        "looks like, then build it in small reviewed steps, and I don't close it until I've seen it "
        "working in production.\"*"),
)

# ---- 3. AI-assisted dev ----------------------------------------------------
page(
    opener("AI-assisted development", "Reviewing code you didn't write",
           "They already use AI tools. The skill they're hiring for is judging the output."),
    why("**Reviewing generated code to ensure changes are well-targeted, maintainable, secure and properly "
        "tested.** Those four words are their checklist; use them back."),
    table(["Their word", "What you check"], [
        ["Well-targeted", "Does the diff only touch what the ticket needs? Agents love to \"improve\" nearby code. Revert anything unrelated."],
        ["Maintainable", "Does it match the codebase's patterns and naming? No new abstractions for one use, no duplicated helpers that already exist."],
        ["Secure", "Parameterised SQL, escaped output, authorisation checks on every route, no secrets in code or logs, input validated."],
        ["Properly tested", "Tests that would fail without the change. Watch for tests that only test mocks, or assertions weakened to pass."],
    ]),
    h(3, "Where AI output goes wrong"),
    bullets([
        "**Confident invention**: a function, column or config option that doesn't exist. Check it does.",
        "**Silent scope creep**: a 40-line fix becomes a 400-line diff.",
        "**Happy path only**: nulls, empty lists, timezones and concurrency get missed.",
        "**Swallowed errors**: `try/catch` that hides the real failure.",
        "**Tests bent to fit**: the test changed to match buggy behaviour, not the other way round.",
    ]),
    h(3, "Improving the workflow itself"),
    bullets([
        "Project instructions (e.g. `CLAUDE.md`) with conventions, commands and \"don't touch\" areas.",
        "Make the agent run the tests and linters before it says it's done.",
        "Reusable prompts or skills for common tickets: a bug fix, a migration, a new endpoint.",
        "Automated review on every PR as a first pass, never a replacement for a human.",
    ]),
    say("*\"I use Claude Code every day, and the more I use it the more I see my job as setting it up "
        "well and reviewing hard. I read every diff as if a new colleague wrote it.\"*"),
)

# ---- 4. RAG ----------------------------------------------------------------
page(
    opener("Helpdesk automation", "How a RAG pipeline works",
           "Retrieval-augmented generation: find the relevant information first, then let the model answer from it."),
    why("Maintaining and developing **their existing AI-powered RAG pipeline** is the most distinctive part "
        "of the job. Expect detailed questions here."),
    steps([
        ("Indexing (ahead of time)", [
            ("Collect sources", "Help articles, past resolved tickets, product docs, release notes, known issues."),
            ("Chunk", "Split into passages of a few hundred tokens, by heading or section, with a little overlap."),
            ("Embed and store", "Turn each chunk into a vector; store it with metadata (product, version, date, source URL)."),
        ]),
        ("Answering (per ticket)", [
            ("Build the query", "From the ticket's subject and body, plus customer context: plan, product, version."),
            ("Retrieve", "Top matches by vector similarity, ideally combined with keyword search (hybrid), filtered by metadata."),
            ("Rerank", "Reorder the candidates with a reranker so the best few go to the model."),
            ("Generate", "The model drafts an answer *only* from the retrieved context, with citations."),
        ]),
    ]),
    table(["Problem", "Fix"], [
        ["Finds the wrong articles", "Better chunking, hybrid search for error codes and product names, metadata filters, reranking."],
        ["Out-of-date answers", "Re-index on change; store dates; prefer newer sources; retire old articles."],
        ["Makes things up", "Answer only from context, require citations, allow \"I don't know\", lower temperature."],
        ["Can't tell if it's improving", "An evaluation set of real past tickets with known good answers; measure every change."],
    ]),
)

page(
    opener("Helpdesk automation", "Drafting, resolution and escalation",
           "The job description says *progressively*. Automate in stages, with a human in the loop until the numbers say otherwise."),
    steps([
        ("Stage 1: Enrich", [
            ("Add context to every ticket", "Customer's account, recent errors from logs, related articles and similar past tickets, for the agent to see."),
            ("Classify", "Category, product area, urgency and sentiment, so tickets reach the right person."),
        ]),
        ("Stage 2: Draft", [
            ("Suggest a reply", "The helpdesk agent edits and sends. Nothing goes to the customer unreviewed."),
            ("Measure", "How often drafts are sent unchanged, lightly edited or discarded. That's your quality signal."),
        ]),
        ("Stage 3: Resolve", [
            ("Auto-send for narrow, safe categories", "Password resets, how-to questions with a clear article, only above a confidence threshold."),
            ("Always offer a human", "Every automated reply says how to reach a person."),
        ]),
        ("Escalate", [
            ("Clear rules", "Low confidence, angry customer, billing or data issues, security, repeated contact, or a likely bug go to a person."),
            ("Hand over with context", "The escalation carries the summary, what was tried and the retrieved sources."),
        ]),
    ]),
    callout("warn", "Risks to raise before they do",
            "**Prompt injection**: ticket text is untrusted input; it must never be able to change the "
            "model's instructions or trigger actions. **Personal data**: what goes to the model provider, "
            "and is it logged? **Wrong answers at scale**: one bad article can now reach hundreds of "
            "customers, so monitor and keep a kill switch."),
    say("*\"I'd start by measuring what the pipeline does today against real past tickets, because you can't "
        "improve retrieval you can't measure. Then automate the narrow, safe cases first and widen it as "
        "the draft-acceptance rate proves it.\"*"),
)

# ---- 5. SQL ----------------------------------------------------------------
page(
    opener("Database management", "SQL you should be fluent in",
           "Strong SQL is the first thing in the candidate profile. Expect a live query or a whiteboard question."),
    why("**Write and maintain SQL, manage database updates, schema changes and migrations, and ensure database "
        "integrity and performance.** Fluency here is non-negotiable."),
    table(["Topic", "What to say"], [
        ["Joins", "INNER vs LEFT; LEFT JOIN … WHERE b.id IS NULL to find missing rows."],
        ["Aggregation", "GROUP BY with HAVING to filter groups; COUNT(DISTINCT …)."],
        ["Window functions", "ROW_NUMBER, RANK, LAG, running totals; MySQL 8 has them."],
        ["CTEs", "WITH … AS for readable multi-step queries; recursive CTEs for trees."],
        ["Indexes", "Composite indexes follow the leftmost-prefix rule; cover the WHERE then the ORDER BY."],
        ["EXPLAIN", "Look for type = ALL (full scan), \"Using filesort\", \"Using temporary\", rows examined."],
        ["Transactions", "InnoDB; BEGIN / COMMIT; default isolation REPEATABLE READ; keep them short."],
        ["Integrity", "Foreign keys, NOT NULL, unique constraints, correct types (DECIMAL for money)."],
    ]),
    code("sql",
         "-- Latest ticket per customer, with how many they've raised\n"
         "WITH ranked AS (\n"
         "    SELECT t.*,\n"
         "           ROW_NUMBER() OVER (PARTITION BY customer_id ORDER BY created_at DESC) AS rn,\n"
         "           COUNT(*)     OVER (PARTITION BY customer_id)                          AS total\n"
         "    FROM tickets t\n"
         "    WHERE created_at >= NOW() - INTERVAL 30 DAY\n"
         ")\n"
         "SELECT customer_id, id AS latest_ticket, subject, total\n"
         "FROM ranked\n"
         "WHERE rn = 1 AND total >= 3\n"
         "ORDER BY total DESC;"),
)

page(
    opener("Database management", "Performance and safe schema changes",
           "How to find a slow query, and how to change a big table without taking the site down."),
    h(3, "Finding a slow query"),
    steps([
        ("Diagnose", [
            ("Find it", "Slow query log, Performance Insights on RDS, or performance_schema."),
            ("Read the plan", "EXPLAIN, or EXPLAIN ANALYZE (MySQL 8.0.18+) for actual timings."),
            ("Fix the cause", "Add or reorder an index, avoid functions on indexed columns, fix N+1 queries in the app, paginate."),
        ]),
    ]),
    code("sql",
         "EXPLAIN ANALYZE\n"
         "SELECT id, subject FROM tickets\n"
         "WHERE status = 'open' AND assigned_to = 42\n"
         "ORDER BY created_at DESC LIMIT 20;\n\n"
         "-- Serves the filter and the sort in one index\n"
         "CREATE INDEX idx_tickets_status_assignee_created\n"
         "    ON tickets (status, assigned_to, created_at);\n\n"
         "-- Not this: a function on the column can't use the index\n"
         "-- WHERE DATE(created_at) = '2026-10-23'\n"
         "-- Do this instead\n"
         "-- WHERE created_at >= '2026-10-23' AND created_at < '2026-10-24'"),
    h(3, "Changing a big table safely"),
    facts([
        ("Expand, then contract", "Add the new column, deploy code that writes both, backfill in batches, switch reads, then drop the old column in a later release."),
        ("Online DDL", "MySQL 8 can add many columns with ALGORITHM=INSTANT, and many index changes with INPLACE, without locking writes."),
        ("Big tables", "pt-online-schema-change or gh-ost copy the table in the background and swap it in."),
        ("Backfills", "In batches of a few thousand rows, with a pause between, so you don't flood replication."),
        ("Always", "A backup or snapshot first, tested on a copy of production data, and a rollback plan."),
    ]),
)

# ---- 6. CI/CD --------------------------------------------------------------
page(
    opener("Deployment and automation", "CI/CD and reliable releases",
           "Maintain and improve their pipelines, and automate the repetitive work."),
    why("**Maintain and improve existing CI/CD pipelines, automate repetitive development tasks and support "
        "reliable software releases.** Note *existing*: understand it before you change it."),
    steps([
        ("A good pipeline", [
            ("On every push", "Install dependencies (cached), lint and static analysis (PHPStan), run the tests."),
            ("On merge", "Build one artefact, deploy it to staging, run smoke tests."),
            ("To production", "The same artefact, with migrations first, then a health check and an easy rollback."),
        ]),
    ]),
    code("yaml",
         "# GitHub Actions: the shape of a PHP pipeline\n"
         "jobs:\n"
         "  test:\n"
         "    runs-on: ubuntu-latest\n"
         "    services:\n"
         "      mysql: { image: mysql:8.0, env: { MYSQL_ROOT_PASSWORD: root, MYSQL_DATABASE: test } }\n"
         "    steps:\n"
         "      - uses: actions/checkout@v4\n"
         "      - uses: shivammathur/setup-php@v2\n"
         "        with: { php-version: '8.3' }\n"
         "      - run: composer install --no-interaction --prefer-dist\n"
         "      - run: vendor/bin/phpstan analyse\n"
         "      - run: vendor/bin/phpunit"),
    table(["Release practice", "Why"], [
        ["Small, frequent deploys", "Less to go wrong; easier to find what did."],
        ["Backward-compatible migrations", "Old and new code both work during the deploy."],
        ["Feature flags", "Ship code dark, turn it on for some customers first."],
        ["Automatic rollback on failed health check", "Bad deploys fix themselves in minutes."],
    ]),
    callout("info", "Automating repetitive tasks",
            "Good examples to offer: generating release notes from merged PRs, setting up a local environment "
            "with one command, refreshing a staging database with anonymised data, and AI review of every PR "
            "as a first pass."),
)

# ---- 7. AWS monitoring -----------------------------------------------------
page(
    opener("Infrastructure monitoring", "AWS monitoring and alerting",
           "Availability, memory, CPU and performance are named explicitly. Know which need extra setup."),
    why("**Implement and maintain AWS monitoring and alerting, covering system availability, memory utilisation, "
        "CPU capacity, performance and potential issues.** This is a gap they want filled."),
    callout("warn", "The detail that shows you've done it",
            "EC2 sends **CPU** to CloudWatch by default, but **not memory or disk space**. Those need the "
            "**CloudWatch agent** installed on the instance. RDS reports memory as **FreeableMemory** without it."),
    table(["Watch", "Metric", "Alert when"], [
        ["Availability", "ALB HealthyHostCount; a Synthetics canary or uptime check", "Any host unhealthy, or the check fails twice"],
        ["Errors", "ALB HTTPCode_Target_5XX_Count", "5xx rate above normal for 5 minutes"],
        ["Performance", "ALB TargetResponseTime (p95, p99)", "p95 above your target"],
        ["CPU", "EC2 / RDS CPUUtilization", "Above 80% for 10 to 15 minutes"],
        ["Memory", "Agent mem_used_percent; RDS FreeableMemory", "Above 85% used, or falling steadily"],
        ["Disk", "Agent disk_used_percent; RDS FreeStorageSpace", "Above 80% used"],
        ["Database", "DatabaseConnections, ReadLatency / WriteLatency, ReplicaLag", "Near max connections, or lag growing"],
        ["Queues", "SQS ApproximateAgeOfOldestMessage", "Messages waiting too long"],
    ]),
    facts([
        ("Route alerts", "CloudWatch alarm → SNS topic → email, Slack or PagerDuty."),
        ("Avoid noise", "Alert on symptoms customers feel; use evaluation periods so a spike doesn't page anyone; composite alarms to combine signals."),
        ("Dashboards", "One CloudWatch dashboard per service: traffic, errors, latency, saturation."),
        ("Logs", "Ship app and PHP error logs to CloudWatch Logs; metric filters turn error lines into alarms."),
    ]),
)

page(
    opener("Your story", "Typeset: memory on a small server",
           "Your own production system, and exactly the kind of issue they want caught early. Rehearse it."),
    facts([
        ("Situation", "Typeset, a PDF render API you built and run in production on a server with 960 MB of RAM."),
        ("Problem", "Large renders ran out of memory and the process was killed (OOM)."),
        ("What you did", "Diagnosed it as memory, not a code bug; added swap as an immediate fix so renders complete."),
        ("What's next", "Limit how many renders run at once and add a timeout, so one big job can't starve the rest."),
        ("The lesson", "Monitor memory, not just CPU, and alert before it runs out, not after. That's the point of the role's monitoring work."),
    ]),
    callout("info", "Make it theirs",
            "Close with how you'd apply it at Workhorse: *\"On AWS I'd install the CloudWatch agent so memory is "
            "visible, alarm at 85%, and look for which requests drive the peaks.\"*"),
    h(3, "Investigating a production issue"),
    steps([
        ("Investigate", [
            ("Scope it", "Who's affected, since when, what changed? Check recent deploys first."),
            ("Gather evidence", "Logs, metrics and the database, around the time it started. Reproduce it if you can."),
            ("Mitigate, then fix", "Roll back or work around to stop the harm; then find and fix the root cause."),
            ("Write it up", "What happened, why, and what stops it happening again. Add the alarm that would have caught it."),
        ]),
    ]),
    callout("good", "Working with the outsourced infrastructure team",
            "Escalate with evidence: the time it started, the metrics, the logs, what you've ruled out. "
            "It saves them hours and earns trust. Learn from each handover so next time you can fix it yourself."),
)

# ---- 8. PHP ----------------------------------------------------------------
page(
    opener("The platform", "PHP refresher",
           "PHP is a bonus for them, which makes it an advantage for you. Be ready to talk about legacy code too."),
    table(["Topic", "What to say"], [
        ["PHP 8 features", "Typed properties, constructor promotion, enums, match, readonly, named arguments, nullsafe ?->."],
        ["Strict types", "declare(strict_types=1) catches type bugs early."],
        ["PDO", "Prepared statements with bound parameters, always. Never build SQL from strings."],
        ["Composer", "Dependencies and PSR-4 autoloading; commit composer.lock."],
        ["Testing", "PHPUnit or Pest; PHPStan for static analysis."],
        ["Legacy code", "Add tests around it before changing it; refactor in small steps; don't rewrite."],
    ]),
    code("php",
         "<?php declare(strict_types=1);\n\n"
         "final class TicketRepository\n"
         "{\n"
         "    public function __construct(private readonly PDO $db) {}\n\n"
         "    /** @return list<array{id: int, subject: string}> */\n"
         "    public function openFor(int $agentId, int $limit = 20): array\n"
         "    {\n"
         "        $stmt = $this->db->prepare(\n"
         "            'SELECT id, subject FROM tickets\n"
         "             WHERE status = :status AND assigned_to = :agent\n"
         "             ORDER BY created_at DESC LIMIT :limit'\n"
         "        );\n"
         "        $stmt->bindValue(':status', 'open');\n"
         "        $stmt->bindValue(':agent', $agentId, PDO::PARAM_INT);\n"
         "        $stmt->bindValue(':limit', $limit, PDO::PARAM_INT);\n"
         "        $stmt->execute();\n\n"
         "        return $stmt->fetchAll(PDO::FETCH_ASSOC);\n"
         "    }\n"
         "}"),
    say("*\"I've worked a lot in PHP and Laravel, so I can be useful in the codebase from the first week, "
        "and I'm comfortable in older code: I'd put tests around something before I change it.\"*"),
)

# ---- 9. stories ------------------------------------------------------------
page(
    opener("Your examples", "Stories to have ready",
           "Use situation, task, action, result. Two minutes each, with a number in the result where you can."),
    table(["Question", "Your story"], [
        ["Something you built and run in production", "Typeset: a render API in production; how you deploy, monitor and fix it."],
        ["A production problem you diagnosed", "The Typeset OOMs: memory not code, swap now, concurrency and timeouts next."],
        ["How you use AI tools", "Claude Code daily: planning first, small diffs, reading every line, tests before done."],
        ["A time AI got it wrong", "Have one specific example: what it invented or broke, and how you caught it."],
        ["Improving a system you didn't build", "A legacy PHP or Laravel app you modernised without a rewrite."],
        ["A slow query you fixed", "The query, what EXPLAIN showed, the index, before and after timings."],
        ["Automating something repetitive", "A script, pipeline or agent workflow that saved you or a team time."],
        ["Helping a non-technical colleague", "Explaining an issue to a client or support person in plain words."],
    ]),
    callout("warn", "Tonight",
            "Fill in the two you're least sure of, especially *a time AI got it wrong* and *a slow query you "
            "fixed*, with real details. Specifics are what make a story believable."),
)

# ---- 10. likely questions --------------------------------------------------
page(
    opener("Preparation", "Questions they're likely to ask"),
    h(3, "Technical"),
    bullets([
        "Walk me through how you'd pick up a Jira ticket you don't fully understand.",
        "How do you review code an AI tool has written? What do you look for?",
        "Explain how a RAG pipeline works. How would you improve one that gives poor answers?",
        "How would you decide when it's safe to send an AI reply to a customer without a person checking it?",
        "What is prompt injection, and how would you protect a helpdesk pipeline from it?",
        "Write a query: customers who raised more than three tickets in the last 30 days.",
        "A page is slow and you suspect the database. What do you do?",
        "How do you add a column to a table with 50 million rows without downtime?",
        "What would you monitor on AWS for a PHP app, and how would you alert on it?",
        "The site is down. Talk me through your first ten minutes.",
    ]),
    h(3, "About you"),
    bullets([
        "Why this role, and why now?",
        "Tell me about something you built that you're proud of.",
        "How do you balance working independently with keeping the team informed?",
        "What excites you about AI, and what worries you about it?",
    ]),
)

# ---- 11. questions to ask --------------------------------------------------
page(
    opener("Preparation", "Questions to ask them",
           "Pick three or four that fit the conversation. They signal that you've thought about the job."),
    h(3, "The helpdesk and RAG pipeline"),
    bullets([
        "What does the RAG pipeline do today, and where does it fall short?",
        "What does it index, and how do you know if its answers are good?",
        "How many tickets a month, and which kinds would you most like automated?",
    ]),
    h(3, "Development"),
    bullets([
        "Which AI development tools do you use, and what's your review process for their output?",
        "What does the path from ticket to production look like? How often do you deploy?",
        "Where is the codebase hardest to work in?",
    ]),
    h(3, "Operations and the team"),
    bullets([
        "What monitoring is in place on AWS now, and what has gone wrong that it didn't catch?",
        "How is the work split with the outsourced infrastructure team?",
        "What would success look like in the first three and six months?",
    ]),
    callout("brand", "Closing line",
            "*\"This role covers everything I enjoy: shipping real changes, making AI tools reliable rather than "
            "just using them, and keeping a production system healthy. I'm used to owning things end to end, "
            "and I'd like to help make the helpdesk automation something the team really trusts.\"*"),
)

# ---- 12. checklist ---------------------------------------------------------
page(
    opener("Before the interview", "Tonight and tomorrow"),
    steps([
        ("Tonight", [
            ("Look them up", "Workhorse's product, customers and recent news; who's interviewing you."),
            ("Rehearse three stories out loud", "Typeset, how you use Claude Code, and a slow query you fixed."),
            ("Write three SQL queries by hand", "A join, a GROUP BY with HAVING, and a window function."),
            ("Explain RAG in two minutes", "Out loud, without notes: index, retrieve, rerank, generate, evaluate."),
        ]),
        ("Tomorrow", [
            ("Re-read the job description", "Its words are your vocabulary: *well-targeted, maintainable, secure, properly tested*."),
            ("Check your setup", "Camera, sound, a working editor and terminal in case of a live exercise."),
            ("Have this guide open", "Plus your questions for them, and a glass of water."),
        ]),
    ]),
)

doc = {
    "schema": 2,
    "frontmatter": {
        "title": "Workhorse",
        "eyebrow": "Interview preparation",
        "subtitle": "Software Engineer – Automation & Operations. PHP, MySQL, AWS and AI-assisted helpdesk automation.",
        "recipient": "Workhorse interview, 23 October",
        "author": "Chris Garlick",
        "date": "22 October 2026",
        "document_type": "brief",
        "client": "chris-garlick-light",
        "cover": "editorial",
    },
    "pages": pages,
}

with open("workhorse-software-engineer.json", "w") as f:
    json.dump(doc, f, indent=2, ensure_ascii=False)
    f.write("\n")
print(f"wrote workhorse-software-engineer.json ({len(pages)} pages)")
