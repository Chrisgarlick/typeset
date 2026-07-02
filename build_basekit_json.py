#!/usr/bin/env python3
"""Generate php-basekit.json from the interview prep content.

Uses the Typeset JSON layout schema (heading/markdown/code/table/columns/section/...).
json.dumps handles all escaping so the output is always valid.
"""
import json

# ---- palette ---------------------------------------------------------------
INK   = "#312E81"   # deep indigo (PHP-ish)
INK_T = "#EEF2FF"
LILAC = "#EEF2FF"   # light indigo card
LILAC_B = "#C7D2FE"
CREAM = "#FAF7F2"
WHITE = "#FFFFFF"
LINE  = "#E2E8F0"
BAD   = "#7F1D1D"
BAD_T = "#FEF2F2"
BAD_B = "#FCA5A5"
GOOD  = "#064E3B"
GOOD_T = "#ECFDF5"

# ---- block helpers ---------------------------------------------------------
def h(level, text):           return {"type": "heading", "level": level, "text": text}
def md(content):              return {"type": "markdown", "content": content}
def code(lang, content):      return {"type": "code", "language": lang, "content": content}
def bullets(items):           return {"type": "bullet_list", "items": items}
def ordered(items):           return {"type": "ordered_list", "items": items}
def table(headers, rows):     return {"type": "table", "headers": headers, "rows": rows}
def quote(text):              return {"type": "blockquote", "text": text}
def rule():                   return {"type": "rule"}

def section(blocks, bg=None, tc=None, border=None, pad="18pt", radius="6pt"):
    b = {"type": "section", "blocks": blocks, "padding": pad, "radius": radius}
    if bg:     b["background"] = bg
    if tc:     b["text_color"] = tc
    if border: b["border"] = border
    return b

def slot(blocks, bg=None, tc=None, border=None, pad="16pt", radius="6pt"):
    s = {"blocks": blocks, "padding": pad, "radius": radius}
    if bg:     s["background"] = bg
    if tc:     s["text_color"] = tc
    if border: s["border"] = border
    return s

def columns(children, ratios=None, gutter="8mm"):
    if ratios is None:
        ratios = [1] * len(children)
    return {"type": "columns", "ratios": ratios, "gutter": gutter, "children": children}

def why(text):
    """The recurring 'Why it matters' callout."""
    return section([h(4, "WHY IT MATTERS"), md(text)], bg=INK, tc=INK_T, pad="16pt")

def page(blocks):
    return {"blocks": blocks}

pages = []

# ===========================================================================
# Page 1 — Contents + how to use
# ===========================================================================
pages.append(page([
    h(2, "About This Guide"),
    md("A focused refresher for the **BaseKit PHP Backend Developer — Round 2** "
       "interview. Each section opens with *why it matters*, then gives the concept "
       "in plain English and a PHP example you can speak to. Read it once end-to-end, "
       "then drill the topics that came up in round one."),
    columns([
        slot([
            h(4, "CORE LANGUAGE"),
            md("1. PHP 8 vs PHP 7\n2. Dependency Injection\n3. SOLID Principles\n"
               "9. OOP Concepts"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
        slot([
            h(4, "DATA & APIs"),
            md("4. MySQL Performance\n5. REST API Design\n10. Security Essentials"),
        ], bg=CREAM),
        slot([
            h(4, "TOOLING & CRAFT"),
            md("6. Docker & Kubernetes\n7. Linux / Mac CLI\n8. Git Workflow\n"
               "11. Best Practices"),
        ], bg=WHITE, border={"width": "1pt", "color": LINE}),
    ], ratios=[1, 1, 1], gutter="6mm"),
    section([
        md("**How to use it:** skim the headings, read every *Why it matters* box, "
           "and rehearse saying each idea out loud. The phrases on the final page are "
           "the ones to drop into answers.")
    ], bg=CREAM),
]))

# ===========================================================================
# 1. PHP 8 Features vs PHP 7
# ===========================================================================
pages.append(page([
    h(2, "1 · PHP 8 Features vs PHP 7"),
    why("PHP 8 (Nov 2020) was a major release — big performance gains and real "
        "language features, built on by 8.1 and 8.2. Coming from WordPress / PHP 7, "
        "these are the things you need to know cold."),

    h(3, "Named Arguments  ·  8.0"),
    md("Pass arguments *by name*, in any order, skipping optional ones."),
    code("php",
"""// PHP 7 — positional only
array_slice($array, 0, 5, true);

// PHP 8 — named, readable, order doesn't matter
array_slice(array: $array, offset: 0, length: 5, preserve_keys: true);"""),

    h(3, "Union Types  ·  8.0"),
    md("Declare that a value can be one of several types."),
    code("php",
"""function processInput(int|string $input): int|string {
    return $input;
}"""),

    h(3, "Nullsafe Operator  ·  8.0"),
    md("Collapses nested null checks — if any link is null, the chain returns null."),
    columns([
        slot([md("**PHP 7**"), code("php",
"""$country = null;
if ($user !== null) {
    if ($user->getAddress() !== null) {
        $country = $user
            ->getAddress()
            ->getCountry();
    }
}""")], bg=CREAM),
        slot([md("**PHP 8**"), code("php",
"""$country = $user
    ?->getAddress()
    ?->getCountry();""")], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),
]))

pages.append(page([
    h(3, "Match Expressions  ·  8.0"),
    md("Like `switch`, but it **returns a value**, uses strict `===` comparison, and "
       "has no fall-through."),
    code("php",
"""$label = match($status) {
    1 => 'Active',
    2 => 'Inactive',
    default => 'Unknown',
};"""),
    md("A `match` with no arm and no `default` throws `UnhandledMatchError` — safer "
       "than `switch` silently doing nothing."),

    h(3, "Constructor Property Promotion  ·  8.0"),
    md("Declare and assign properties in one line — great for DTOs and value objects."),
    code("php",
"""class User {
    public function __construct(
        private string $name,
        private string $email,
    ) {}
}"""),

    h(3, "Readonly Properties  ·  8.1"),
    md("Set once at initialisation, immutable thereafter."),
    code("php",
"""class Order {
    public function __construct(
        public readonly int $id,
        public readonly string $status,
    ) {}
}

$order = new Order(1, 'pending');
$order->id = 2;  // Error: Cannot modify readonly property"""),
]))

pages.append(page([
    h(3, "Enums  ·  8.1"),
    md("Native enumerations — no more class constants or magic strings."),
    code("php",
"""enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
}

function setStatus(Status $status): void { /* ... */ }
setStatus(Status::Active);

$status = Status::from('active'); // Status::Active"""),

    columns([
        slot([
            h(4, "FIBERS · 8.1"),
            md("Lightweight concurrency — pause and resume execution without a full "
               "event loop. Know *what* they are and *why* they exist; you won't be "
               "asked to write one."),
        ], bg=INK, tc=INK_T),
        slot([
            h(4, "FIRST-CLASS CALLABLES · 8.1"),
            md("Cleaner way to pass a function as a callable:"),
            code("php", "$fn = strlen(...);\narray_map(strlen(...), $strings);"),
        ], bg=CREAM),
    ], ratios=[1, 1]),

    h(3, "Intersection Types  ·  8.1   &   readonly Classes  ·  8.2"),
    md("Union = *this OR that*; intersection = *this AND that* (must satisfy all). "
       "A `readonly` class makes **every** property readonly by default."),
    code("php",
"""function process(Iterator&Countable $collection): void { /* ... */ }

readonly class Point {
    public function __construct(public float $x, public float $y) {}
}"""),
]))

# ===========================================================================
# 2. Dependency Injection
# ===========================================================================
pages.append(page([
    h(2, "2 · Dependency Injection"),
    why("BaseKit's job ads call out DI as a core practice, and they use it heavily on "
        "the React frontend too. Being able to articulate *why* DI is good — "
        "testability, loose coupling, single responsibility — matters as much as the how."),
    section([
        md("**Plain English:** instead of a class creating its own dependencies, you "
           "*pass them in from outside*. The class declares what it needs; something "
           "else provides it.\n\nLike a restaurant kitchen — the chef doesn't go to "
           "the farm, ingredients are delivered. The chef just cooks.")
    ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),

    h(3, "The problem it solves"),
    columns([
        slot([
            h(4, "✗ WITHOUT DI"),
            md("Tightly coupled, hard to test."),
            code("php",
"""class OrderService {
    public function __construct() {
        $this->db = new MySQLDatabase();
        $this->mailer = new SmtpMailer();
    }
}
// Can't test without a real DB + SMTP"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ WITH DI"),
            md("Loosely coupled, testable."),
            code("php",
"""class OrderService {
    public function __construct(
        private DatabaseInterface $db,
        private MailerInterface $mailer,
    ) {}
}
// Tests pass in fakes/mocks"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
]))

pages.append(page([
    h(3, "Three types of injection"),
    columns([
        slot([
            h(4, "CONSTRUCTOR"),
            md("Most common — required upfront."),
            code("php", "public function __construct(\n    private LoggerInterface $logger,\n) {}"),
        ], bg=CREAM),
        slot([
            h(4, "SETTER"),
            md("Optional — set after construction."),
            code("php", "public function setLogger(\n    LoggerInterface $logger,\n): void {\n    $this->logger = $logger;\n}"),
        ], bg=CREAM),
        slot([
            h(4, "INTERFACE"),
            md("The dependency supplies an injector method. Rare in practice."),
        ], bg=WHITE, border={"width": "1pt", "color": LINE}),
    ], ratios=[1, 1, 1], gutter="6mm"),

    h(3, "DI Containers"),
    md("Real apps wire everything together with a container — Laravel's service "
       "container, Symfony's DI component, or PHP-DI."),
    code("php",
"""// Bind an interface to a concrete implementation
$container->bind(DatabaseInterface::class, MySQLDatabase::class);

// Resolving OrderService now injects a MySQLDatabase automatically
$service = $container->make(OrderService::class);"""),
    section([
        md("**Say this:** *\"DI gives me loose coupling and testability — I program "
           "to interfaces, not implementations.\"*")
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 3. SOLID
# ===========================================================================
pages.append(page([
    h(2, "3 · SOLID Principles"),
    why("BaseKit mention SOLID explicitly. Know each letter and a one-line PHP example "
        "for it — the goal of all five is small, focused classes that are easy to change."),

    h(3, "S — Single Responsibility"),
    md("A class should have one, and only one, reason to change."),
    code("php",
"""// GOOD — each concern is separate
class UserRepository { public function save(User $u): void {} }
class UserMailer     { public function sendWelcome(User $u): void {} }
class UserReporter   { public function generate(User $u): void {} }"""),

    h(3, "O — Open/Closed"),
    md("Open for extension, closed for modification — add behaviour without editing "
       "existing code."),
    code("php",
"""interface PaymentGateway {
    public function charge(float $amount): void;
}
class StripeGateway implements PaymentGateway { /* ... */ }
class PaypalGateway implements PaymentGateway { /* ... */ }

class PaymentProcessor {
    public function process(PaymentGateway $gateway, float $amount): void {
        $gateway->charge($amount);   // new types need no changes here
    }
}"""),
]))

pages.append(page([
    h(3, "L — Liskov Substitution"),
    md("Subtypes must be usable anywhere their base type is, without breaking things. "
       "The classic trap: `Square extends Rectangle` forces both dimensions equal and "
       "breaks the rectangle's contract. Prefer a shared interface when behaviour differs."),
    code("php",
"""// Square::setWidth() also sets height — a surprise side effect that breaks LSP.
// Instead, share an interface:
interface Shape {
    public function area(): float;
}"""),

    h(3, "I — Interface Segregation"),
    md("Don't force classes to implement methods they don't use. Many small "
       "interfaces beat one fat one."),
    code("php",
"""interface Workable  { public function work(): void; }
interface Breakable { public function takeBreak(): void; }

class HumanWorker implements Workable, Breakable { /* ... */ }
class RobotWorker implements Workable { /* robots don't need breaks */ }"""),

    h(3, "D — Dependency Inversion"),
    md("High-level modules shouldn't depend on low-level ones — both depend on "
       "abstractions."),
    code("php",
"""interface DatabaseInterface { public function query(string $sql): array; }

class OrderService {
    public function __construct(private DatabaseInterface $db) {}
}
class MySQLDatabase  implements DatabaseInterface { /* ... */ }
class SQLiteDatabase implements DatabaseInterface { /* ... */ }"""),
]))

# ===========================================================================
# 4. MySQL Performance
# ===========================================================================
pages.append(page([
    h(2, "4 · MySQL Query Performance"),
    why("This came up in round one — expect more in round two. Lead with EXPLAIN, "
        "then talk indexes, then N+1."),

    h(3, "EXPLAIN — the first move"),
    md("Run it before optimising any query."),
    code("sql",
"""EXPLAIN SELECT * FROM orders WHERE user_id = 42;

-- MySQL 8+: actual timings, not estimates
EXPLAIN ANALYZE SELECT * FROM orders WHERE user_id = 42;"""),
    columns([
        slot([
            h(4, "READING IT"),
            md("- **type** — `const` → `ref` → `range` → `index` → `ALL` (worst). "
               "`ALL` = full table scan.\n"
               "- **key** — index used (NULL = none).\n"
               "- **rows** — estimated rows examined.\n"
               "- **Extra** — beware `Using filesort` / `Using temporary`."),
        ], bg=CREAM),
        slot([
            h(4, "SAY THIS"),
            quote("First thing I'd do is run EXPLAIN and check whether we're doing a "
                  "full table scan."),
        ], bg=INK, tc=INK_T),
    ], ratios=[1, 1]),

    h(3, "Indexes"),
    md("A separate structure that finds rows without scanning the whole table — like "
       "a book's index. **Leftmost-prefix rule** governs composite indexes."),
    code("sql",
"""CREATE INDEX idx_user_status ON orders(user_id, status);
-- helps:  WHERE user_id = 1 AND status = 'active'
-- helps:  WHERE user_id = 1
-- NO help: WHERE status = 'active'   (no leftmost prefix)"""),
    md("**When indexes hurt:** every index slows writes (INSERT/UPDATE/DELETE). Don't "
       "index blindly."),
]))

pages.append(page([
    h(3, "The N+1 Problem"),
    md("One of the most common performance killers, especially in ORMs."),
    columns([
        slot([
            h(4, "✗ N+1 — 501 QUERIES"),
            code("php",
"""$orders = Order::all();   // 1 query
foreach ($orders as $order) {
    echo $order->user->name; // +1 each
}"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ EAGER LOAD — 2 QUERIES"),
            code("php",
"""$orders = Order::with('user')->get();
foreach ($orders as $order) {
    echo $order->user->name;
}"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
    md("In raw SQL, JOIN instead of looping:"),
    code("sql",
"""SELECT orders.*, users.name
FROM orders
JOIN users ON orders.user_id = users.id;"""),

    h(3, "Slow Query Log"),
    code("sql",
"""SET GLOBAL slow_query_log = 'ON';
SET GLOBAL long_query_time = 1;   -- log queries over 1 second"""),
    code("bash",
"""# Summarise: sort by time, top 10
mysqldumpslow -s t -t 10 /var/log/mysql/mysql-slow.log"""),
]))

pages.append(page([
    h(3, "Other Common Bottlenecks"),
    columns([
        slot([
            h(4, "SELECT *"),
            md("Fetches columns you don't need. Select only what you use."),
            code("sql", "-- GOOD\nSELECT id, name, email\nFROM users WHERE id = 1;"),
        ], bg=CREAM),
        slot([
            h(4, "LEADING WILDCARD"),
            md("`LIKE '%x%'` can't use an index. Use a FULLTEXT index for search."),
            code("sql", "ALTER TABLE products\n  ADD FULLTEXT(name);\nSELECT * FROM products\nWHERE MATCH(name)\n  AGAINST('widget');"),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    md("**Missing WHERE on large tables** — always filter early. **Transactions** — "
       "wrap related writes so they commit together and stay consistent:"),
    code("sql",
"""START TRANSACTION;
INSERT INTO orders (user_id, total) VALUES (1, 99.99);
INSERT INTO order_items (order_id, product_id) VALUES (LAST_INSERT_ID(), 5);
COMMIT;"""),
]))

# ===========================================================================
# 5. REST API Design
# ===========================================================================
pages.append(page([
    h(2, "5 · REST API Design"),
    section([
        h(4, "CORE PRINCIPLES"),
        md("- Resources are **nouns**, not verbs\n"
           "- Use HTTP **methods** to express intent\n"
           "- **Stateless** — each request carries everything it needs"),
    ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),

    columns([
        slot([
            h(4, "METHODS & ROUTES"),
            code("text",
"""GET    /users      list
GET    /users/42   fetch one
POST   /users      create
PUT    /users/42   replace
PATCH  /users/42   partial update
DELETE /users/42   delete"""),
        ], bg=CREAM),
        slot([
            h(4, "STATUS CODES"),
            code("text",
"""200 OK          success
201 Created     POST created
204 No Content  DELETE ok
400 Bad Request invalid data
401 Unauthorized not auth'd
403 Forbidden   not allowed
404 Not Found
422 Unprocessable validation
429 Too Many    rate limited
500 Server Error"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),

    h(3, "Versioning"),
    md("Always version so you can ship breaking changes without breaking clients: "
       "`/api/v1/users`, `/api/v2/users`."),

    h(3, "Clean endpoint (Laravel)"),
    code("php",
"""public function show(int $id): JsonResponse
{
    $user = $this->userRepository->findById($id);

    if (!$user) {
        return response()->json(['error' => 'User not found'], 404);
    }

    return response()->json($user, 200);
}"""),
]))

# ===========================================================================
# 6. Docker & Kubernetes
# ===========================================================================
pages.append(page([
    h(2, "6 · Docker & Containerisation"),
    why("BaseKit use Docker and Kubernetes. You don't need to be a DevOps expert — "
        "know the basics and the vocabulary."),
    columns([
        slot([
            h(4, "KEY CONCEPTS"),
            md("- **Image** — a blueprint (like a class)\n"
               "- **Container** — a running instance (like an object)\n"
               "- **Dockerfile** — build instructions\n"
               "- **docker-compose** — run many containers together"),
        ], bg=INK, tc=INK_T),
        slot([
            h(4, "ESSENTIAL COMMANDS"),
            code("bash",
"""docker ps              # running
docker ps -a           # all
docker logs -f myapp   # tail logs
docker exec -it myapp bash
docker build -t app:latest .
docker run -d -p 8080:80 nginx"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),

    h(3, "Typical PHP Dockerfile"),
    code("dockerfile",
"""FROM php:8.2-fpm
RUN docker-php-ext-install pdo pdo_mysql
WORKDIR /var/www
COPY . .
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer
RUN composer install --no-dev --optimize-autoloader
EXPOSE 9000
CMD [\"php-fpm\"]"""),
]))

pages.append(page([
    h(3, "docker-compose"),
    code("yaml",
"""services:
  app:
    build: .
    ports: ["8000:8000"]
    depends_on: [db]
    environment:
      DB_HOST: db
  db:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: secret
      MYSQL_DATABASE: myapp
    volumes:
      - db_data:/var/lib/mysql
volumes:
  db_data:"""),
    code("bash",
"""docker-compose up -d      # start all
docker-compose down       # stop all
docker-compose logs -f    # tail all"""),

    section([
        h(4, "KUBERNETES — HIGH LEVEL"),
        md("Orchestrates containers at scale. Know the four nouns:\n\n"
           "- **Pod** — smallest unit, one or more containers\n"
           "- **Deployment** — rolling updates and replicas\n"
           "- **Service** — exposes pods to network traffic\n"
           "- **Ingress** — routes external traffic to services"),
    ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
]))

# ===========================================================================
# 7. Linux / Mac CLI
# ===========================================================================
pages.append(page([
    h(2, "7 · Linux & Mac Command Line"),
    md("The commands that come up day-to-day as a backend dev. Mac and Linux both use "
       "bash/zsh."),
    columns([
        slot([
            h(4, "NAVIGATE & FILES"),
            code("bash",
"""pwd
ls -la
cd /var/www/html
tail -f logfile.log
grep -r "findme" ./
find . -name "*.php" """),
        ], bg=CREAM),
        slot([
            h(4, "PROCESSES"),
            code("bash",
"""ps aux | grep php
kill 1234
kill -9 1234     # force
top
htop"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    columns([
        slot([
            h(4, "PERMISSIONS"),
            code("bash",
"""chmod 755 script.sh
chmod +x script.sh
chown www-data:www-data /var/www
sudo command"""),
        ], bg=CREAM),
        slot([
            h(4, "NETWORKING"),
            code("bash",
"""curl -I https://example.com
wget https://host/file.zip
netstat -tulpn
lsof -i :8080"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
]))

pages.append(page([
    columns([
        slot([
            h(4, "SSH & COPY"),
            code("bash",
"""ssh user@host
ssh -i ~/.ssh/key user@host
scp file.txt user@host:/path"""),
        ], bg=CREAM),
        slot([
            h(4, "SHORTCUTS"),
            code("bash",
"""Ctrl+C   kill process
Ctrl+R   search history
!!       repeat last command
!$       last arg of prev cmd"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    section([
        h(4, "PHP & COMPOSER"),
        code("bash",
"""php -v                  # version
php -m                  # modules
php -r "echo phpversion();"
php artisan migrate     # Laravel
composer install
composer dump-autoload"""),
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 8. Git
# ===========================================================================
pages.append(page([
    h(2, "8 · Git Workflow"),
    columns([
        slot([
            h(4, "EVERYDAY"),
            code("bash",
"""git status
git diff
git add .
git commit -m "message"
git push origin feature/x
git pull origin main
git log --oneline --graph"""),
        ], bg=CREAM),
        slot([
            h(4, "BRANCHING"),
            code("bash",
"""git checkout -b feature/new
git checkout main
git merge feature/new
git rebase main
git branch -d feature/new"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    columns([
        slot([
            h(4, "FIXING MISTAKES"),
            code("bash",
"""git stash / git stash pop
git reset HEAD~1        # keep changes
git reset --hard HEAD~1 # DISCARD
git revert abc1234      # safe on shared"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "COMMIT MESSAGE FORMAT"),
            code("text",
"""feat: add auth endpoint
fix: null pointer in processor
refactor: extract payment service
chore: bump composer deps"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# 9. OOP
# ===========================================================================
pages.append(page([
    h(2, "9 · OOP Concepts"),
    h(3, "The four pillars"),
    columns([
        slot([
            h(4, "ENCAPSULATION"),
            md("Hide internal state; expose only what's needed."),
            code("php",
"""class BankAccount {
    private float $balance = 0;
    public function deposit(float $a): void {
        if ($a <= 0) throw new InvalidArgumentException();
        $this->balance += $a;
    }
}"""),
        ], bg=CREAM),
        slot([
            h(4, "ABSTRACTION"),
            md("Expose *what* something does, not *how*."),
            code("php",
"""interface Cache {
    public function get(string $k): mixed;
    public function set(string $k, mixed $v, int $ttl): void;
}
// Redis, Memcached, file — same interface"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    columns([
        slot([
            h(4, "INHERITANCE"),
            md("Child classes extend parent behaviour."),
            code("php",
"""abstract class Notification {
    abstract public function send(string $m): void;
    public function log(string $m): void { /* ... */ }
}
class EmailNotification extends Notification {
    public function send(string $m): void { /* ... */ }
}"""),
        ], bg=CREAM),
        slot([
            h(4, "POLYMORPHISM"),
            md("Same call, different behaviour per class."),
            code("php",
"""foreach ($notifications as $n) {
    $n->send("Hello!");
    // each type sends differently
}"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
]))

pages.append(page([
    h(3, "Composition over Inheritance"),
    why("BaseKit explicitly prefer this. Instead of inheriting behaviour, inject it — "
        "flatter hierarchies, easier testing."),
    columns([
        slot([
            h(4, "✗ INHERITANCE — RIGID"),
            code("php",
"""class LoggingOrderService
    extends OrderService {
    public function create(): void {
        $this->log("Creating order");
        parent::create();
    }
}"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ COMPOSITION — FLEXIBLE"),
            code("php",
"""class OrderService {
    public function __construct(
        private OrderRepository $repo,
        private LoggerInterface $logger,
    ) {}
    public function create(): void {
        $this->logger->info("Creating order");
        $this->repo->save();
    }
}"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
    section([
        md("**Say this:** *\"I prefer composing behaviour through injected dependencies "
           "over deep inheritance hierarchies.\"*")
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 10. Security
# ===========================================================================
pages.append(page([
    h(2, "10 · Security Essentials"),
    h(3, "SQL Injection"),
    md("Never interpolate user input into SQL. Use prepared statements."),
    columns([
        slot([
            h(4, "✗ VULNERABLE"),
            code("php",
"""$id = $_GET['id']; // 1 OR 1=1
$db->query(
  "SELECT * FROM users WHERE id = $id"
);"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ SAFE"),
            code("php",
"""$stmt = $db->prepare(
  "SELECT * FROM users WHERE id = ?"
);
$stmt->execute([$id]);"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),

    h(3, "XSS — escape output"),
    code("php",
"""// SAFE
echo "<p>Hello " . htmlspecialchars($_GET['name'], ENT_QUOTES, 'UTF-8') . "</p>";"""),

    columns([
        slot([
            h(4, "CSRF"),
            md("Use tokens to prove a form came from your own site. Laravel does this "
               "automatically with `@csrf`."),
        ], bg=CREAM),
        slot([
            h(4, "PASSWORD HASHING"),
            md("Never plain text, MD5 or SHA1."),
            code("php",
"""$hash = password_hash($pw, PASSWORD_BCRYPT);
if (password_verify($pw, $hash)) { /* ok */ }"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    section([
        h(4, "SECRETS"),
        md("Never commit secrets. Keep them in `.env`, and put `.env` in `.gitignore`."),
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 11. Best Practices
# ===========================================================================
pages.append(page([
    h(2, "11 · General Best Practices"),
    columns([
        slot([
            h(4, "CODE REVIEW MINDSET"),
            md("- Readable in 6 months?\n- Is it tested?\n- Edge cases handled?\n"
               "- Is there a simpler way?"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
        slot([
            h(4, "THREE ACRONYMS"),
            md("- **DRY** — extract repeated logic\n"
               "- **YAGNI** — don't build for needs that don't exist\n"
               "- **KISS** — simplest thing that works; complexity must be earned"),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    h(3, "Testing basics (PHPUnit)"),
    code("php",
"""class OrderServiceTest extends TestCase {
    public function test_creates_order_successfully(): void {
        $mockRepo = $this->createMock(OrderRepository::class);
        $mockRepo->expects($this->once())->method('save');

        $service = new OrderService($mockRepo);
        $service->create(['user_id' => 1, 'total' => 50.00]);
    }
}"""),
    section([
        md("*If something is hard to test, it's usually a sign the design needs "
           "revisiting.*")
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# Part Two divider
# ===========================================================================
pages.append(page([
    h(2, "Part Two · Deeper Technical Topics"),
    section([
        md("The first half is the breadth refresher. This half goes deeper into the "
           "things a *technical* round tends to probe — language internals, data "
           "integrity, caching, async, and how a PHP app actually runs and scales."),
    ], bg=INK, tc=INK_T, pad="22pt"),
    columns([
        slot([
            h(4, "LANGUAGE DEPTH"),
            md("12. Type System & Gotchas\n13. Arrays & Higher-Order Fns\n"
               "14. Traits / Abstract / Interface\n15. Generators & Closures\n"
               "16. Errors & Exceptions"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
        slot([
            h(4, "ARCHITECTURE & DATA"),
            md("17. Design Patterns\n18. PSR & Composer\n19. ACID & Isolation\n"
               "20. Caching Strategies\n21. Queues & Background Jobs"),
        ], bg=CREAM),
        slot([
            h(4, "RUNTIME & SCALE"),
            md("22. Auth & HTTP\n23. Request Lifecycle / FPM\n24. Testing Deeper\n"
               "25. Big-O & Coding\n26. Scaling & Multi-Tenancy"),
        ], bg=WHITE, border={"width": "1pt", "color": LINE}),
    ], ratios=[1, 1, 1], gutter="6mm"),
]))

# ===========================================================================
# 12. Type System & Gotchas
# ===========================================================================
pages.append(page([
    h(2, "12 · Type System & Gotchas"),
    why("\"What does this print?\" is a classic technical-round opener. Type juggling "
        "and loose comparison catch out anyone who hasn't looked closely at PHP's "
        "rules — and PHP 8 quietly changed several of them."),

    h(3, "== vs ===  (loose vs strict)"),
    md("`==` coerces types before comparing; `===` checks value **and** type. Default "
       "to `===` unless you *want* coercion."),
    code("php",
"""var_dump(0 == "a");     // PHP 7: true (!)   PHP 8: false
var_dump("1" == 1);      // true  — numeric string
var_dump("abc" == 0);    // PHP 7: true        PHP 8: false
var_dump(null == false); // true
var_dump([] == false);   // true
var_dump(0 === "0");     // false — different types"""),
    md("**The PHP 8 fix:** comparing a number to a non-numeric string now casts the "
       "*number* to string, so `0 == \"abc\"` is finally `false`. Knowing this one "
       "signals you've actually used PHP 8."),

    h(3, "Handy operators"),
    code("php",
"""$name = $data['name'] ?? 'guest';   // null coalescing — no warning if unset
$config['x'] ??= 'default';          // assign only if null/unset
$cmp = $a <=> $b;                    // spaceship: -1, 0, or 1
declare(strict_types=1);             // top of file — enforce type declarations"""),
    section([
        md("**Say this:** *\"I put `declare(strict_types=1)` at the top of every file "
           "and use `===` by default — type juggling should be a deliberate choice, "
           "not an accident.\"*")
    ], bg=CREAM),
]))

# ===========================================================================
# 13. Arrays & Higher-Order Functions
# ===========================================================================
pages.append(page([
    h(2, "13 · Arrays & Higher-Order Functions"),
    md("PHP arrays are ordered maps — they double as lists, dictionaries, stacks and "
       "queues. Knowing the functional trio cold reads as fluency."),
    code("php",
"""$nums = [1, 2, 3, 4];

$doubled = array_map(fn($n) => $n * 2, $nums);            // [2, 4, 6, 8]
$evens   = array_filter($nums, fn($n) => $n % 2 === 0);   // [1 => 2, 3 => 4]
$sum     = array_reduce($nums, fn($c, $n) => $c + $n, 0); // 10"""),
    columns([
        slot([
            h(4, "GOTCHA — KEYS"),
            md("`array_filter` **preserves keys**, leaving gaps. Reindex with "
               "`array_values()` before sending as a JSON array."),
            code("php", "$evens = array_values(\n  array_filter($nums, $isEven)\n);"),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "WORTH KNOWING"),
            md("- `array_column()` — pluck a field\n"
               "- `array_combine()` — keys + values\n"
               "- `usort()` — custom sort with `<=>`\n"
               "- `[...$a, ...$b]` — spread/merge"),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    md("**`array_map` vs `foreach`:** `array_map` is declarative and returns a new "
       "array (no mutation). Reach for a `foreach` when you need the index, early "
       "exit, or side effects."),
]))

pages.append(page([
    h(2, "13 · Arrays on Real Data"),
    md("The trio shines on rows from the database. Say you've fetched a list of orders:"),
    code("php",
"""$orders = [
    ['id' => 1, 'user' => 'Ann', 'total' => 30, 'paid' => true],
    ['id' => 2, 'user' => 'Bob', 'total' => 80, 'paid' => false],
    ['id' => 3, 'user' => 'Ann', 'total' => 50, 'paid' => true],
];"""),
    columns([
        slot([
            h(4, "PLUCK A COLUMN"),
            code("php",
"""$ids = array_column($orders, 'id');
// [1, 2, 3]

// key it by id:
$byId = array_column(
    $orders, null, 'id'
);"""),
        ], bg=CREAM),
        slot([
            h(4, "SUM THE PAID ONES"),
            code("php",
"""$revenue = array_sum(
    array_column(
        array_filter(
            $orders,
            fn($o) => $o['paid']
        ),
        'total'
    )
); // 80"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    h(3, "Grouping (the one people forget)"),
    md("There's no `array_group` — you build it with a `foreach`. Worth knowing by heart:"),
    code("php",
"""$byUser = [];
foreach ($orders as $o) {
    $byUser[$o['user']][] = $o;   // append into a bucket keyed by user
}
// 'Ann' => [order 1, order 3], 'Bob' => [order 2]"""),
    md("**Sorting by a field** uses the spaceship operator inside `usort`:"),
    code("php",
"""usort($orders, fn($a, $b) => $b['total'] <=> $a['total']);  // highest total first"""),
]))

# ===========================================================================
# 14. Traits, Abstract Classes & Interfaces
# ===========================================================================
pages.append(page([
    h(2, "14 · Traits, Abstract & Interface"),
    why("\"When would you use a trait over an abstract class?\" tests whether you "
        "understand composition, contracts and PHP's single-inheritance model."),
    columns([
        slot([
            h(4, "INTERFACE"),
            md("A **contract** — no implementation. A class can implement many. "
               "*\"Can do X.\"*"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
        slot([
            h(4, "ABSTRACT CLASS"),
            md("Partial implementation + shared state. **Single** inheritance, can't "
               "be instantiated. *\"Is a kind of X.\"*"),
        ], bg=CREAM),
        slot([
            h(4, "TRAIT"),
            md("Horizontal **code reuse** — copy-pasted in at compile time. Use for "
               "shared behaviour across unrelated classes."),
        ], bg=WHITE, border={"width": "1pt", "color": LINE}),
    ], ratios=[1, 1, 1], gutter="6mm"),
    code("php",
"""interface Notifier { public function send(string $msg): void; }

abstract class BaseRepository {
    abstract protected function table(): string;   // subclass must define
    public function all(): array { /* uses $this->table() */ }
}

trait Timestamps {
    public function touch(): void { $this->updatedAt = new DateTime(); }
}
class User { use Timestamps; }"""),

    h(3, "self vs static (late static binding)"),
    code("php",
"""class Model {
    public static function create(): static {
        return new static();   // resolves to the CALLED class, not Model
    }
}
// User::create() returns a User, not a Model"""),
]))

# ===========================================================================
# 15. Generators & Closures
# ===========================================================================
pages.append(page([
    h(2, "15 · Generators & Closures"),
    h(3, "Generators — lazy iteration"),
    md("`yield` produces values one at a time without building the whole array in "
       "memory. The interview angle: *processing a huge file or result set without "
       "running out of memory*."),
    code("php",
"""function readLines(string $file): Generator {
    $h = fopen($file, 'r');
    while (($line = fgets($h)) !== false) {
        yield trim($line);     // one line in memory at a time
    }
    fclose($h);
}

foreach (readLines('huge.log') as $line) {
    // constant memory, even for a 10GB file
}"""),

    h(3, "Closures & captured scope"),
    code("php",
"""$tax = 0.2;

// Arrow fn — auto-captures by value
$withTax = fn(float $net) => $net * (1 + $tax);

// Classic closure — explicit capture; & captures by reference
$total = 0;
$add = function (int $n) use (&$total) { $total += $n; };"""),
    section([
        md("**Key distinction:** arrow functions (`fn`) capture outer variables "
           "automatically *by value*; classic `function () use (...)` closures capture "
           "explicitly, and `&` makes the capture by reference.")
    ], bg=CREAM),
]))

# ===========================================================================
# 16. Errors & Exceptions
# ===========================================================================
pages.append(page([
    h(2, "16 · Errors & Exceptions"),
    md("Everything throwable implements `Throwable`, which splits into **`Error`** "
       "(engine-level: `TypeError`, `DivisionByZeroError`) and **`Exception`** "
       "(application-level, the ones you throw and catch)."),
    code("php",
"""try {
    $receipt = $payments->charge($order);
} catch (InsufficientFundsException $e) {
    // catch the SPECIFIC case first
    return $this->declineGracefully($e);
} catch (PaymentException | NetworkException $e) {   // PHP 8 union catch
    $logger->error('Charge failed', ['order' => $order->id, 'e' => $e]);
    throw new OrderFailedException('Could not place order', previous: $e);
} finally {
    $lock->release();   // always runs — cleanup
}"""),
    columns([
        slot([
            h(4, "CUSTOM EXCEPTIONS"),
            code("php",
"""class InsufficientFundsException
    extends DomainException {}"""),
            md("Extend `DomainException`, `RuntimeException`, etc. so callers can catch "
               "by type."),
        ], bg=CREAM),
        slot([
            h(4, "DON'T DO THIS"),
            md("- Swallow exceptions with an empty `catch`\n"
               "- Catch `\\Exception` everywhere\n"
               "- Use exceptions for normal control flow\n"
               "- Lose the original via `previous`"),
        ], bg=BAD, tc=BAD_T),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# 17. Design Patterns in PHP
# ===========================================================================
pages.append(page([
    h(2, "17 · Design Patterns"),
    why("BaseKit value clean architecture. You don't need to recite the Gang of Four "
        "— but naming the right pattern for a problem, and knowing one anti-pattern, "
        "lands well."),
    table(
        ["Pattern", "Use it to…"],
        [
            ["Repository", "Hide persistence behind an interface — swap MySQL for a fake in tests."],
            ["Factory", "Centralise complex object creation; return an interface, not a concrete."],
            ["Strategy", "Swap interchangeable behaviours (e.g. payment gateways) without if-ladders."],
            ["Observer", "Let many listeners react to an event (e.g. 'order placed') without coupling."],
            ["Adapter", "Wrap a third-party API in your own interface so vendor changes stay contained."],
            ["Decorator", "Add behaviour (logging, caching) by wrapping, not subclassing."],
        ],
    ),
    columns([
        slot([
            h(4, "STRATEGY — THE GO-TO"),
            code("php",
"""interface ShippingRate {
    public function cost(Order $o): float;
}
class Standard implements ShippingRate { /* ... */ }
class Express  implements ShippingRate { /* ... */ }

// Inject the chosen strategy — no switch statement
$checkout = new Checkout($express);"""),
        ], bg=CREAM),
        slot([
            h(4, "SINGLETON — ⚠ ANTI-PATTERN"),
            md("Hidden global state, hard to test, hides dependencies. In modern PHP "
               "the **DI container** manages a single shared instance for you — "
               "without the downsides. Mention this trade-off if asked."),
        ], bg=BAD, tc=BAD_T),
    ], ratios=[1, 1]),
]))

pages.append(page([
    h(2, "17 · Patterns in Code"),
    h(3, "Repository — hide the database"),
    md("Callers depend on an *interface*, not on MySQL. Swap a fake in tests instantly."),
    code("php",
"""interface UserRepository {
    public function find(int $id): ?User;
    public function save(User $user): void;
}

class MySqlUserRepository implements UserRepository { /* real SQL */ }
class InMemoryUserRepository implements UserRepository { /* array, for tests */ }

// The service neither knows nor cares which one it got
class Onboarding {
    public function __construct(private UserRepository $users) {}
}"""),
    h(3, "Factory — centralise tricky creation"),
    code("php",
"""class GatewayFactory {
    public function make(string $type): PaymentGateway {
        return match ($type) {
            'stripe' => new StripeGateway(config('stripe.key')),
            'paypal' => new PaypalGateway(config('paypal.token')),
            default  => throw new InvalidArgumentException("Unknown: $type"),
        };
    }
}"""),
    h(3, "Observer — react without coupling"),
    md("The order doesn't know who's listening; listeners subscribe to the event."),
    code("php",
"""$dispatcher->listen(OrderPlaced::class, fn($e) => $mailer->sendReceipt($e->order));
$dispatcher->listen(OrderPlaced::class, fn($e) => $warehouse->reserve($e->order));

// Later, the order just announces what happened:
$dispatcher->dispatch(new OrderPlaced($order));"""),
]))

# ===========================================================================
# 18. PSR & Composer
# ===========================================================================
pages.append(page([
    h(2, "18 · PSR Standards & Composer"),
    md("PSRs are the shared conventions that let PHP packages interoperate. Know what "
       "the common ones cover."),
    columns([
        slot([
            h(4, "THE ONES TO NAME"),
            md("- **PSR-4** — autoloading (namespace → path)\n"
               "- **PSR-12** — coding style\n"
               "- **PSR-3** — logger interface\n"
               "- **PSR-7 / 15** — HTTP messages & middleware\n"
               "- **PSR-11** — container interface"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
        slot([
            h(4, "SEMVER"),
            md("`^7.2` → any `7.x` (≥ 7.2)\n`~7.2` → `7.2.x` only\n\n"
               "`MAJOR.MINOR.PATCH` — major = breaking, minor = features, patch = fixes."),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    h(3, "composer.json"),
    code("json",
"""{
  "require": {
    "php": "^8.2",
    "guzzlehttp/guzzle": "^7.0"
  },
  "autoload": {
    "psr-4": { "App\\\\": "src/" }
  }
}"""),
    code("bash",
"""composer require monolog/monolog            # add a dependency
composer install --no-dev --optimize-autoloader   # production install
composer dump-autoload                       # rebuild the autoloader"""),
]))

# ===========================================================================
# 19. Transactions, ACID & Isolation
# ===========================================================================
pages.append(page([
    h(2, "19 · ACID & Isolation Levels"),
    why("Money, bookings, anything that must not half-happen — this is where backend "
        "rigour shows. Expect questions on transactions, locking and race conditions."),
    columns([
        slot([
            h(4, "ACID"),
            md("- **Atomic** — all or nothing\n- **Consistent** — valid state → valid "
               "state\n- **Isolated** — concurrent txns don't interfere\n- **Durable** "
               "— committed data survives a crash"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
        slot([
            h(4, "SAFE WRITE"),
            code("php",
"""$db->beginTransaction();
try {
    $db->debit($from, 100);
    $db->credit($to, 100);
    $db->commit();
} catch (Throwable $e) {
    $db->rollBack();
    throw $e;
}"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    h(3, "Isolation levels (weakest → strongest)"),
    table(
        ["Level", "Allows", "Notes"],
        [
            ["READ UNCOMMITTED", "Dirty reads", "Rarely used"],
            ["READ COMMITTED", "Non-repeatable reads", "Postgres default"],
            ["REPEATABLE READ", "Phantom reads*", "MySQL/InnoDB default"],
            ["SERIALIZABLE", "Nothing — fully isolated", "Safest, slowest"],
        ],
    ),
    md("Use `SELECT ... FOR UPDATE` to lock rows you're about to change and avoid lost "
       "updates. *InnoDB's gap locking largely prevents phantoms even at REPEATABLE READ.*"),
]))

pages.append(page([
    h(2, "19 · A Race Condition in Action"),
    why("This is the classic *\"two requests at the same time\"* question. If you can "
        "describe the bug **and** the fix, you sound like someone who's run production."),
    md("Imagine the last ticket to a gig. Two requests check stock at the same instant:"),
    columns([
        slot([
            h(4, "✗ THE BUG — LOST UPDATE"),
            code("php",
"""// Both requests read stock = 1
$stock = $db->query(
  "SELECT stock FROM events WHERE id=7"
);
if ($stock > 0) {
    // both pass the check...
    $db->query(
      "UPDATE events
       SET stock = stock - 1
       WHERE id = 7"
    );
}
// Sold 2 tickets. Stock is now -1."""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ THE FIX — LOCK THE ROW"),
            code("php",
"""$db->beginTransaction();
// FOR UPDATE makes the 2nd request
// WAIT until the 1st commits
$stock = $db->query(
  "SELECT stock FROM events
   WHERE id = 7 FOR UPDATE"
);
if ($stock > 0) {
    $db->query(
      "UPDATE events
       SET stock = stock - 1
       WHERE id = 7"
    );
}
$db->commit();"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
    section([
        md("**Even simpler:** push the check into the write itself — "
           "`UPDATE events SET stock = stock - 1 WHERE id = 7 AND stock > 0` — then "
           "check the affected-row count. The database does the locking for you, and "
           "stock can never go negative.")
    ], bg=CREAM),
]))

# ===========================================================================
# 20. Caching Strategies
# ===========================================================================
pages.append(page([
    h(2, "20 · Caching Strategies"),
    md("*\"There are only two hard things in computer science: cache invalidation and "
       "naming things.\"* Know the layers and the cache-aside pattern."),
    columns([
        slot([
            h(4, "THE LAYERS"),
            md("- **OPcache** — compiled PHP bytecode (turn on in prod)\n"
               "- **App cache** — Redis / Memcached for data\n"
               "- **HTTP cache** — `Cache-Control`, ETags, CDN\n"
               "- **Query cache** — materialised/computed results"),
        ], bg=INK, tc=INK_T),
        slot([
            h(4, "CACHE-ASIDE"),
            code("php",
"""function getUser(int $id): array {
    $key = "user:$id";
    if ($hit = $redis->get($key)) {
        return json_decode($hit, true);
    }
    $user = $db->find($id);
    $redis->setex($key, 3600, json_encode($user));
    return $user;
}"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    md("**Invalidation:** set a sensible **TTL**, and bust the key on write "
       "(`$redis->del(\"user:$id\")` after an update). **Write-through** updates cache "
       "and DB together; **cache-aside** (above) is the common default. Always have a "
       "plan for a cold cache — a thundering herd of misses can take the DB down."),
]))

# ===========================================================================
# 21. Queues & Background Jobs
# ===========================================================================
pages.append(page([
    h(2, "21 · Queues & Background Jobs"),
    why("Anything slow — sending email, resizing images, calling a third-party API — "
        "shouldn't block the HTTP response. Knowing why and how to defer work is a "
        "core backend signal."),
    md("A **producer** pushes a job onto a **broker** (Redis, RabbitMQ, SQS); a "
       "separate **worker** process pulls and runs it asynchronously."),
    code("php",
"""// Producer — returns immediately, user isn't kept waiting
SendWelcomeEmail::dispatch($user)->onQueue('emails');

// Worker (run as a long-lived process / container)
//   php artisan queue:work --queue=emails --tries=3"""),
    columns([
        slot([
            h(4, "GET RIGHT"),
            md("- **Idempotent** jobs — safe to run twice\n"
               "- **Retries** with backoff\n"
               "- **Dead-letter queue** for poison jobs\n"
               "- Pass IDs, not whole objects"),
        ], bg=CREAM),
        slot([
            h(4, "WHY IT MATTERS"),
            md("Faster responses, resilience to spikes (the queue absorbs load), and "
               "failures that retry instead of losing data. Workers scale "
               "independently of web servers."),
        ], bg=INK, tc=INK_T),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# 22. Auth & HTTP
# ===========================================================================
pages.append(page([
    h(2, "22 · API Auth & HTTP"),
    columns([
        slot([
            h(4, "SESSION (STATEFUL)"),
            md("Server stores the session; client holds a cookie. Easy to revoke; "
               "needs shared session storage (Redis) when scaled."),
        ], bg=CREAM),
        slot([
            h(4, "JWT (STATELESS)"),
            md("Signed token holds the claims: `header.payload.signature`. Nothing "
               "stored server-side — but hard to revoke before expiry. Keep them "
               "short-lived + use refresh tokens."),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),
    code("text",
"""Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI0MiJ9.<signature>"""),
    h(3, "Things they may poke at"),
    md("- **OAuth2** — delegated access; *authentication* (who you are) vs "
       "*authorization* (what you can do).\n"
       "- **CORS** — the browser's pre-flight `OPTIONS` check; a server header, not a "
       "security boundary.\n"
       "- **Idempotency** — `GET`, `PUT`, `DELETE` are idempotent; `POST` is not. "
       "Matters for safe retries.\n"
       "- **Always HTTPS**, never log tokens, hash anything secret at rest."),
]))

# ===========================================================================
# 23. Request Lifecycle & PHP-FPM
# ===========================================================================
pages.append(page([
    h(2, "23 · Request Lifecycle & PHP-FPM"),
    why("A favourite senior question: *\"What actually happens when a request hits a "
        "PHP app?\"* The shared-nothing model explains a lot of PHP's behaviour."),
    section([
        h(4, "THE PATH OF A REQUEST"),
        md("**nginx** receives the request → forwards to a **PHP-FPM** pool over "
           "FastCGI → an FPM **worker** bootstraps the app, handles the request, sends "
           "the response → the worker is **reset** and reused for the next request."),
    ], bg=INK, tc=INK_T),
    columns([
        slot([
            h(4, "SHARED-NOTHING"),
            md("Every request starts with a **clean slate** — no variables persist "
               "between requests (unlike Node). So global in-memory state is useless "
               "across requests."),
        ], bg=CREAM),
        slot([
            h(4, "CONSEQUENCES"),
            md("- Sessions/cache live in **Redis or the DB**, not memory\n"
               "- **OPcache** keeps compiled bytecode hot\n"
               "- Statelessness makes **horizontal scaling** trivial"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),
    md("**Say this:** *\"PHP is shared-nothing — each request is isolated, which is "
       "why state goes in Redis or the database and why we can scale just by adding "
       "more FPM workers behind the load balancer.\"*"),
]))

# ===========================================================================
# 24. Testing Deeper
# ===========================================================================
pages.append(page([
    h(2, "24 · Testing, Deeper"),
    columns([
        slot([
            h(4, "THE PYRAMID"),
            md("Many **unit** tests (fast, isolated), fewer **integration** tests "
               "(real DB), fewest **end-to-end** (slow, brittle). Optimise the bulk "
               "for speed."),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
        slot([
            h(4, "AAA + TDD"),
            md("**Arrange, Act, Assert** per test. **TDD** = red → green → refactor: "
               "write a failing test, make it pass, then clean up."),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    h(3, "Test doubles & data providers"),
    md("**Stub** returns canned data; **mock** also asserts it was called; **fake** is "
       "a working lightweight implementation (e.g. in-memory repo); **spy** records "
       "calls. Data providers run one test over many cases:"),
    code("php",
"""#[DataProvider('vatCases')]
public function test_vat_is_added(float $net, float $expected): void {
    $this->assertSame($expected, addVat($net));
}

public static function vatCases(): array {
    return [
        'standard' => [100.0, 120.0],
        'zero'     => [0.0, 0.0],
    ];
}"""),
    md("Coverage is a *guide*, not a goal — 100% coverage of trivial getters proves "
       "little; one good test of the money path proves a lot."),
]))

# ===========================================================================
# 25. Big-O — what it means
# ===========================================================================
pages.append(page([
    h(2, "25 · Big-O Notation"),
    why("Big-O is just a way of answering one question: *as the input gets bigger, "
        "how fast does the work grow?* It's not about seconds — it's about shape. "
        "Interviewers use it to check you can reason about cost before you write code."),
    section([
        md("**The mental model.** `n` is the size of the input (number of items). "
           "Big-O describes the *number of steps* relative to `n`. We ignore constants "
           "and only keep the dominant term — `O(2n + 5)` is just **O(n)**, because "
           "when `n` is huge the rest stops mattering.")
    ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),

    h(3, "Why it matters — the numbers"),
    md("This table is the whole point. It shows roughly how many steps each class "
       "takes as the input grows. Watch what O(n²) does."),
    table(
        ["Input size n", "O(1)", "O(log n)", "O(n)", "O(n log n)", "O(n²)"],
        [
            ["10",    "1", "3",  "10",    "33",     "100"],
            ["100",   "1", "7",  "100",   "664",    "10,000"],
            ["1,000", "1", "10", "1,000", "9,966",  "1,000,000"],
            ["1,000,000", "1", "20", "1 million", "~20 million", "1 trillion ☠"],
        ],
    ),
    md("**O(1)** doesn't care how big the input is. **O(log n)** barely grows — a "
       "million items takes ~20 steps. **O(n²)** explodes — a million items is a "
       "*trillion* steps, i.e. it will never finish. That's the difference."),
]))

pages.append(page([
    h(2, "25 · A Code Example for Each"),
    md("Same idea, one function per class. The thing to notice is the **loop shape**."),

    columns([
        slot([
            h(4, "O(1) · CONSTANT"),
            md("No loop — jump straight to the answer. Same cost for 10 or 10 million."),
            code("php",
"""$first  = $users[0];        // index
$exists = isset($map[$key]); // hash"""),
        ], bg=CREAM),
        slot([
            h(4, "O(n) · LINEAR"),
            md("One loop over the input — touch each item once."),
            code("php",
"""$total = 0;
foreach ($orders as $o) {
    $total += $o->amount;
}"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),

    h(3, "O(log n) · logarithmic — halve the problem each step"),
    md("Binary search on a **sorted** array. Each step throws away half of what's "
       "left, so a million items takes only ~20 steps."),
    code("php",
"""function binarySearch(array $sorted, int $target): int {
    $lo = 0; $hi = count($sorted) - 1;
    while ($lo <= $hi) {
        $mid = intdiv($lo + $hi, 2);
        if ($sorted[$mid] === $target) return $mid;   // found
        if ($sorted[$mid] < $target) $lo = $mid + 1;   // discard left half
        else                          $hi = $mid - 1;   // discard right half
    }
    return -1;
}"""),

    h(3, "O(n log n) · the cost of a good sort"),
    code("php",
"""sort($nums);                                   // ~O(n log n)
usort($users, fn($a, $b) => $a->age <=> $b->age);"""),
]))

pages.append(page([
    h(2, "25 · The O(n²) Trap & The Fix"),
    md("**O(n²) · quadratic** — a loop *inside* a loop over the same data. This is the "
       "one to watch for: it looks innocent and dies on real data."),
    columns([
        slot([
            h(4, "✗ O(n²) — NESTED LOOPS"),
            md("Compares every item against every other — 1,000 items = 1,000,000 "
               "comparisons."),
            code("php",
"""// Has this array got duplicates?
foreach ($items as $i => $a) {
    foreach ($items as $j => $b) {
        if ($i !== $j && $a === $b) {
            return true;
        }
    }
}"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ O(n) — ONE PASS + HASH"),
            md("Remember what you've seen in a hash map — 1,000 items = 1,000 steps."),
            code("php",
"""$seen = [];
foreach ($items as $a) {
    if (isset($seen[$a])) {
        return true;
    }
    $seen[$a] = true;
}
return false;"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),

    section([
        h(4, "HOW TO SPOT IT AT A GLANCE"),
        md("- **No loop**, direct access → **O(1)**\n"
           "- **Halving** the search space each step → **O(log n)**\n"
           "- **One loop** over the input → **O(n)**\n"
           "- A **sort**, or divide-and-conquer → **O(n log n)**\n"
           "- A **loop inside a loop** over the same data → **O(n²)** ⚠"),
    ], bg=INK, tc=INK_T),

    h(3, "The interview move"),
    md("Trading nested loops for a hash map — **O(n²) → O(n)** — is the single most "
       "common optimisation they look for. The classic *two-sum* shows it: instead of "
       "checking every pair, remember each number and look up its complement."),
    code("php",
"""function twoSum(array $nums, int $target): ?array {
    $seen = [];
    foreach ($nums as $i => $n) {
        if (isset($seen[$target - $n])) {       // O(1) lookup, not a second loop
            return [$seen[$target - $n], $i];
        }
        $seen[$n] = $i;
    }
    return null;
}"""),
    section([
        md("**Say this:** *\"The brute force is O(n²) with nested loops. I can trade "
           "memory for speed — a hash map of what I've seen gets it to O(n) in a single "
           "pass.\"* That sentence is what they're listening for.")
    ], bg=CREAM),
]))

# ===========================================================================
# 26. Scaling & Multi-Tenancy
# ===========================================================================
pages.append(page([
    h(2, "26 · Scaling & Multi-Tenancy"),
    why("BaseKit is a white-label website builder sold through partners — a "
        "multi-tenant SaaS serving many customers from shared infrastructure. Showing "
        "you understand *their* shape of problem is a real edge."),
    columns([
        slot([
            h(4, "SCALING BASICS"),
            md("- **Vertical** — bigger box (limited)\n"
               "- **Horizontal** — more boxes behind a **load balancer** (the PHP way)\n"
               "- **Stateless app servers** make this trivial\n"
               "- **Read replicas** for read-heavy load\n"
               "- **CDN** for static assets"),
        ], bg=CREAM),
        slot([
            h(4, "MULTI-TENANCY MODELS"),
            md("- **Shared DB + `tenant_id`** — simplest, must scope *every* query\n"
               "- **Schema per tenant** — stronger isolation\n"
               "- **DB per tenant** — strongest, heaviest to run"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),
    section([
        md("**The trap to mention:** in a shared-database model, a single query that "
           "forgets its `tenant_id` scope leaks one customer's data to another. A "
           "global query scope (or row-level security) makes that the *default*, not "
           "something every developer must remember.")
    ], bg=BAD, tc=BAD_T),
]))

# ===========================================================================
# Part Three divider
# ===========================================================================
pages.append(page([
    h(2, "Part Three · Running PHP in Production"),
    section([
        md("Parts One and Two cover the language and the patterns. This part is the "
           "*operational* half — the questions a senior backend round leans on hardest: "
           "how do you make a database fast, find out **why** a page is slow, and run "
           "the same code reliably on a server that you only ever ran on your laptop."),
    ], bg=INK, tc=INK_T, pad="22pt"),
    columns([
        slot([
            h(4, "DATA AT SPEED"),
            md("27. Database Optimisation, Deeper\n"
               "28. Diagnosing Slow Page Loads\n"
               "29. Connections & Big Result Sets"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
        slot([
            h(4, "THE RUNTIME"),
            md("30. PHP Versions & Performance\n"
               "31. PHP-FPM & OPcache Tuning"),
        ], bg=CREAM),
        slot([
            h(4, "ENVIRONMENTS"),
            md("32. Local vs Staging vs Production\n"
               "33. Observability & Debugging Prod"),
        ], bg=WHITE, border={"width": "1pt", "color": LINE}),
    ], ratios=[1, 1, 1], gutter="6mm"),
]))

# ===========================================================================
# 27. Database Optimisation, Deeper
# ===========================================================================
pages.append(page([
    h(2, "27 · Database Optimisation, Deeper"),
    why("Section 4 was the refresher — EXPLAIN, indexes, N+1. This goes a level down: "
        "how indexes actually work, when an index *won't* be used, and the schema-level "
        "moves that fix slow reads. This is where a senior answer separates from a junior one."),

    h(3, "How an index actually works"),
    md("InnoDB stores the table itself as a **B-tree** keyed on the primary key — the "
       "*clustered index*. A **secondary index** is a second B-tree whose leaves hold "
       "the PK, so a lookup by a secondary index is two hops: find the PK in the index, "
       "then fetch the row from the clustered index."),
    columns([
        slot([
            h(4, "WHY THIS MATTERS"),
            md("- The PK sits inside **every** secondary index — keep it small "
               "(a `BIGINT`, not a random UUID string).\n"
               "- A query needing columns not in the index does a **second lookup** per "
               "row. A *covering index* avoids that.\n"
               "- Random PKs (UUIDv4) cause **page splits** and fragmentation on insert."),
        ], bg=CREAM),
        slot([
            h(4, "COVERING INDEX"),
            md("If the index holds *every* column the query needs, MySQL answers from "
               "the index alone — no row fetch. EXPLAIN shows `Using index`."),
            code("sql",
"""CREATE INDEX idx_cover
  ON orders (user_id, status, total);
-- SELECT total FROM orders
-- WHERE user_id=1 AND status='paid'
-- answered from the index alone"""),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),

    h(3, "When an index won't be used"),
    md("Knowing these saves hours of head-scratching:"),
    code("sql",
"""-- ✗ function on the column → index ignored
WHERE YEAR(created_at) = 2024;
-- ✓ rewrite as a range so the index works
WHERE created_at >= '2024-01-01' AND created_at < '2025-01-01';

-- ✗ leading wildcard → no index
WHERE name LIKE '%smith';
-- ✗ type mismatch → silent cast, index skipped
WHERE phone = 07911123456;   -- phone is VARCHAR; quote it
-- ✗ low selectivity → optimiser prefers a scan
WHERE is_active = 1;          -- if 95% of rows are active"""),
]))

pages.append(page([
    h(2, "27 · Schema & Query Moves"),
    h(3, "Composite index column order"),
    md("Order columns **equality first, then range, then sort**. The optimiser stops "
       "using the index at the first range condition, so put ranges last."),
    code("sql",
"""-- Query: WHERE tenant_id = ? AND status = ? AND created_at > ? ORDER BY created_at
CREATE INDEX idx ON orders (tenant_id, status, created_at);
--                            equality    equality  range + sort
-- Satisfies the filter AND the sort — no 'Using filesort'."""),

    h(3, "Reading EXPLAIN like a senior"),
    table(
        ["Column", "Want to see", "Red flag"],
        [
            ["type", "const, eq_ref, ref, range", "ALL (full scan)"],
            ["key", "the index you expect", "NULL"],
            ["rows", "small vs table size", "millions"],
            ["filtered", "high %", "low % = scan then discard"],
            ["Extra", "Using index", "Using filesort / temporary"],
        ],
    ),

    h(3, "Schema-level fixes for slow reads"),
    columns([
        slot([
            h(4, "DENORMALISE A COUNTER"),
            md("Don't `COUNT(*)` a million rows on every page load. Keep a "
               "`comment_count` column, updated in the **same transaction** as the write."),
        ], bg=CREAM),
        slot([
            h(4, "RIGHT-SIZE TYPES"),
            md("`INT` not `BIGINT` where it fits; `VARCHAR(50)` not `(255)`; `TIMESTAMP` "
               "(4 bytes) over `DATETIME` (8). Smaller rows → more per page → fewer disk reads."),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    md("**Partitioning & archiving:** past tens of millions of rows, partition by date "
       "or move cold rows to an archive table so the hot working set stays small. "
       "**Read replicas** take read load off the primary; route reads to a replica, "
       "writes to the primary."),
    section([
        md("**Say this:** *\"Before adding an index I run EXPLAIN, check selectivity, "
           "and order composite columns equality-first. I'll reach for a covering index "
           "or a denormalised count before I reach for more hardware.\"*")
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 28. Diagnosing Slow Page Loads
# ===========================================================================
pages.append(page([
    h(2, "28 · Diagnosing Slow Page Loads"),
    why("\"A page is slow — how do you find out why?\" is one of the most revealing "
        "questions they ask. The weak answer guesses and starts optimising. The strong "
        "answer **measures first**, isolates the layer, then fixes the biggest cost."),
    section([
        h(4, "THE METHOD — MEASURE, DON'T GUESS"),
        md("1. **Reproduce** and time it end-to-end (browser dev tools → Network).\n"
           "2. **Split the time:** server (TTFB) or front end (assets, JS)?\n"
           "3. On the server, **profile** to find the hot spot — DB? external API? CPU?\n"
           "4. Fix the **biggest** cost first. Re-measure. Repeat."),
    ], bg=INK, tc=INK_T),
    columns([
        slot([
            h(4, "WHERE TIME ACTUALLY GOES"),
            md("- **Database** — too many queries (N+1), or one slow query\n"
               "- **External APIs** — a synchronous call to a third party\n"
               "- **No caching** — recomputing the same thing every request\n"
               "- **PHP CPU** — a tight loop, O(n²), huge (de)serialization\n"
               "- **Front end** — unoptimised images, render-blocking JS"),
        ], bg=CREAM),
        slot([
            h(4, "FIRST QUESTIONS"),
            md("- *How many queries does this page run?* (often the answer)\n"
               "- *Slow for everyone, or one tenant with more data?*\n"
               "- *Slow always, or only under load?*\n"
               "- *Did it just start, or always been slow?*"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),
]))

pages.append(page([
    h(2, "28 · The Tools"),
    md("Name these and you sound like someone who's debugged real performance problems."),
    columns([
        slot([
            h(4, "PROFILERS"),
            md("- **Xdebug profiler** — function-level timings (view in KCachegrind)\n"
               "- **Blackfire / Tideways** — production-grade call graphs; wall time vs "
               "CPU vs I/O\n"
               "- **`microtime(true)`** — quick manual timing of a suspect block"),
        ], bg=CREAM),
        slot([
            h(4, "APM & QUERY VISIBILITY"),
            md("- **New Relic / Datadog APM** — transaction traces in prod\n"
               "- **Laravel Debugbar / Clockwork** — query count + timings per request\n"
               "- **MySQL slow query log** + `EXPLAIN` on the offenders"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),
    h(3, "Quick-and-dirty timing"),
    code("php",
"""$start = microtime(true);
$result = $expensiveThing();
$ms = (microtime(true) - $start) * 1000;
$logger->info('expensiveThing', ['ms' => round($ms, 1)]);

// Count queries in dev (Laravel)
DB::enableQueryLog();
// ... run the page ...
logger()->info('queries', ['n' => count(DB::getQueryLog())]);"""),
    section([
        md("**Say this:** *\"I profile before I change anything — Blackfire or the slow "
           "query log to find the actual hot spot, rather than optimising the thing I "
           "*assume* is slow. Nine times out of ten it's query count, not raw CPU.\"*")
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 29. Connections & Big Result Sets
# ===========================================================================
pages.append(page([
    h(2, "29 · Connections & Big Result Sets"),
    why("Two things quietly kill PHP apps at scale: opening too many database "
        "connections, and loading too many rows into memory at once. Both have clean fixes."),

    h(3, "Connection handling — the PHP reality"),
    md("PHP is **shared-nothing**: each FPM worker opens its own DB connection, and "
       "there's no built-in connection pool like a long-running Node or Java app. So "
       "*connections scale with FPM workers* — and that bites."),
    columns([
        slot([
            h(4, "THE MATH THAT BITES"),
            md("50 FPM workers × 3 app servers = **150 connections**, even idle. "
               "MySQL's `max_connections` default is 151. Add a traffic spike and you "
               "hit *\"Too many connections\"* — an outage."),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "THE FIX"),
            md("Put a pooler in front — **ProxySQL** (MySQL) or **PgBouncer** "
               "(Postgres). It multiplexes many app connections onto a few DB ones. "
               "Size the FPM pool deliberately; don't just crank it up."),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
    md("**Persistent connections** (`PDO::ATTR_PERSISTENT`) reuse a connection across "
       "requests in the same worker — fewer handshakes, but they hold a slot and can "
       "leak transaction/session state. Prefer an external pooler."),

    h(3, "Don't load a million rows"),
    md("`Model::all()` on a big table pulls every row into memory, then PHP hits "
       "`memory_limit` and dies. Stream instead:"),
    code("php",
"""// ✗ loads everything into memory
foreach (Order::all() as $o) { export($o); }

// ✓ chunk — N rows at a time, constant memory
Order::chunkById(1000, function ($orders) {
    foreach ($orders as $o) { export($o); }
});

// ✓ or a lazy cursor (a generator under the hood)
foreach (Order::lazy() as $o) { export($o); }"""),
    md("Same idea in raw PHP: a **generator** (`yield`) or an **unbuffered query** "
       "streams rows instead of buffering the whole set. And always **paginate** list "
       "endpoints — `LIMIT`/`OFFSET`, or keyset pagination for deep pages."),
]))

# ===========================================================================
# 30. PHP Versions & Performance
# ===========================================================================
pages.append(page([
    h(2, "30 · PHP Versions & Performance"),
    why("\"Why upgrade PHP?\" has a great one-line answer: *it's free performance*. "
        "Each modern release got materially faster while your code stayed the same. "
        "Knowing the timeline and what changed signals you keep stacks current."),
    table(
        ["Version", "Released", "Why it matters"],
        [
            ["7.0", "2015", "The big one — ~2x faster than 5.6, half the memory (new engine)."],
            ["7.4", "2019", "Preloading, typed properties, arrow fns. End of the 7.x line (EOL)."],
            ["8.0", "2020", "JIT, named args, match, union types, attributes."],
            ["8.1", "2021", "Enums, readonly props, fibers, first-class callables."],
            ["8.2", "2022", "readonly classes, DNF types, no more dynamic properties."],
            ["8.3 / 8.4", "2023 / 24", "Typed constants, #[Override], property hooks (8.4)."],
        ],
    ),
    columns([
        slot([
            h(4, "THE JIT — BE HONEST"),
            md("PHP 8's **JIT** compiles hot paths to machine code. It helps "
               "**CPU-bound** work (image processing, maths) a lot — but most web apps "
               "are **I/O-bound** (waiting on DB and network), so JIT's real-world web "
               "gain is modest. Saying *that* shows you understand it, not just that it exists."),
        ], bg=CREAM),
        slot([
            h(4, "STAYING CURRENT"),
            md("- Each version: ~2 yrs active support + 1 yr security\n"
               "- Running EOL PHP = unpatched security holes\n"
               "- Upgrades are mostly cheap; **Rector** automates much of the migration\n"
               "- Test against the new version in CI before bumping prod"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),
    section([
        md("**Say this:** *\"Upgrading PHP is some of the cheapest performance you can "
           "buy — 7.0 roughly doubled throughput. I'd keep us on a supported version, "
           "use Rector to automate the migration, and gate it behind CI.\"*")
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 31. PHP-FPM & OPcache Tuning
# ===========================================================================
pages.append(page([
    h(2, "31 · PHP-FPM & OPcache Tuning"),
    why("Section 23 covered the request lifecycle. This is the operational follow-up: "
        "the handful of settings that decide how much traffic one box can take. You "
        "won't be asked to memorise them — knowing what they do is the signal."),

    h(3, "OPcache — turn it on, always"),
    md("Without OPcache, PHP recompiles every `.php` file on every request. OPcache "
       "keeps the compiled **bytecode** in shared memory, so compilation happens once. "
       "It's the single biggest free win in production."),
    code("ini",
"""opcache.enable=1
opcache.memory_consumption=256        ; MB for cached bytecode
opcache.max_accelerated_files=20000   ; raise for large codebases
opcache.validate_timestamps=0         ; prod: don't stat files — clear on deploy"""),
    md("**Gotcha:** with `validate_timestamps=0`, PHP won't notice changed files — you "
       "**must** clear OPcache (or restart FPM) on every deploy, or you serve stale code."),

    h(3, "FPM pool sizing"),
    columns([
        slot([
            h(4, "THE KEY DIALS"),
            code("ini",
"""pm = dynamic
pm.max_children = 20
pm.start_servers = 4
pm.min_spare_servers = 2
pm.max_spare_servers = 6
pm.max_requests = 500"""),
        ], bg=CREAM),
        slot([
            h(4, "HOW TO SIZE IT"),
            md("`max_children ≈ available RAM ÷ avg process memory`. Too high → "
               "swapping (worse than queueing). `max_requests` recycles workers to curb "
               "memory leaks. More workers ≠ more speed if the **DB** is the bottleneck."),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),
    md("**`memory_limit`** is *per request*, not per server — one request trying to "
       "load a million rows hits it and dies (by design). Raising it hides the real "
       "problem; stream the data instead (see §29)."),
]))

# ===========================================================================
# 32. Local vs Staging vs Production
# ===========================================================================
pages.append(page([
    h(2, "32 · Local vs Staging vs Production"),
    why("\"It works on my machine\" is the oldest bug in the world. The senior answer is "
        "**environment parity** — making local, staging and prod as identical as "
        "possible — and keeping the *differences* in config, never in code."),
    columns([
        slot([
            h(4, "WHY THEY DIFFER"),
            md("- Different PHP version or extensions\n"
               "- Different DB version / collation\n"
               "- Far more data, real concurrency in prod\n"
               "- Debug on locally, off in prod\n"
               "- File paths, timezones, locale"),
        ], bg=CREAM),
        slot([
            h(4, "HOW TO KEEP PARITY"),
            md("- **Docker** — same image everywhere (§6)\n"
               "- **`.env` per environment** — config, not code\n"
               "- Same migrations run in every env\n"
               "- Staging mirrors prod (anonymised data)\n"
               "- Pin versions in `composer.json` + lock file"),
        ], bg=LILAC, border={"width": "1pt", "color": LILAC_B}),
    ], ratios=[1, 1]),
    h(3, "Config belongs in the environment"),
    md("The **Twelve-Factor** rule: anything that differs between environments — "
       "credentials, hostnames, feature flags, debug level — lives in environment "
       "variables, never committed. The *same* build artifact ships to every "
       "environment; only the config changes."),
    code("php",
"""// Read from the environment, with a safe default
$debug  = (bool) ($_ENV['APP_DEBUG'] ?? false);
$dbHost = $_ENV['DB_HOST'] ?? '127.0.0.1';

// In Laravel/Symfony this is wrapped:
$debug = config('app.debug');   // env('APP_DEBUG') under the hood"""),
    section([
        md("**The prod-only settings that matter:** `display_errors=Off` (log, don't "
           "show), `APP_DEBUG=false`, OPcache timestamps off, real mail driver, "
           "HTTPS-only cookies. A debug page leaking a stack trace is a *security* bug.")
    ], bg=BAD, tc=BAD_T),
]))

# ===========================================================================
# 33. Observability & Debugging Prod
# ===========================================================================
pages.append(page([
    h(2, "33 · Observability & Debugging Prod"),
    why("You can't attach a step-debugger to production. The senior skill is making the "
        "system *tell you* what's wrong — through logs, metrics and traces — and having "
        "a calm method when it does break."),
    columns([
        slot([
            h(4, "THE THREE PILLARS"),
            md("- **Logs** — discrete events (structured JSON, with context)\n"
               "- **Metrics** — numbers over time (req/s, error rate, p95 latency)\n"
               "- **Traces** — one request's path across services (APM)"),
        ], bg=INK, tc=INK_T),
        slot([
            h(4, "LOG WELL"),
            md("- Use **PSR-3 levels** (debug → emergency) meaningfully\n"
               "- **Structured** logs (JSON) you can actually query\n"
               "- Add a **correlation / request ID** to follow one request\n"
               "- **Never** log secrets, tokens or full card numbers"),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    code("php",
"""// Structured, contextual logging (Monolog / PSR-3)
$logger->error('Payment failed', [
    'order_id'   => $order->id,
    'gateway'    => 'stripe',
    'request_id' => $requestId,
    // no card data, no secrets
]);"""),
    h(3, "A calm method when prod breaks"),
    md("1. **Stop the bleeding** — roll back or feature-flag off first; diagnose after.\n"
       "2. **Check what changed** — recent deploy? traffic spike? dependency down?\n"
       "3. **Read the signals** — error rate, latency graph, logs around the spike.\n"
       "4. **Reproduce** in staging, fix, add a test so it can't recur.\n"
       "5. **Blameless postmortem** — fix the system, not the person."),
    section([
        md("**Say this:** *\"In production I lean on structured logs, metrics and "
           "tracing rather than guesswork. My first instinct in an incident is to roll "
           "back and stop the bleeding, then diagnose calmly from the signals.\"*")
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# Quick Reference + Closing
# ===========================================================================
pages.append(page([
    h(2, "Quick Reference · Phrases to Use"),
    md("Drop these into your answers — they signal you think about *why*, not just *how*."),
    table(
        ["Topic", "Key phrase"],
        [
            ["DI", "Loose coupling and testability — program to interfaces, not implementations."],
            ["SOLID", "Each principle keeps classes small, focused and easy to change."],
            ["N+1", "Eager load relationships rather than lazy loading in a loop."],
            ["Slow queries", "First thing I'd do is run EXPLAIN and check for a full table scan."],
            ["Composition", "I prefer composing behaviour through injected dependencies over deep inheritance."],
            ["== vs ===", "I default to === and strict_types — coercion should be deliberate."],
            ["Transactions", "Wrap related writes so they all commit or all roll back — ACID."],
            ["Caching", "Cache-aside with a TTL; the hard part is invalidation on write."],
            ["Shared-nothing", "Each request starts fresh, so state lives in Redis or the DB — and we scale out."],
            ["Queues", "Push slow work to a background worker; keep jobs idempotent."],
            ["Multi-tenancy", "Scope every query by tenant by default, not by memory."],
            ["Testing", "If something is hard to test, the design usually needs revisiting."],
            ["Indexes", "Covering and composite, equality-first — verify with EXPLAIN before adding."],
            ["Slow page", "Measure and profile first; usually it's query count, not raw CPU."],
            ["Connections", "Connections scale with FPM workers — pool with ProxySQL / PgBouncer."],
            ["Big result sets", "Chunk or stream with a cursor; never load a million rows into memory."],
            ["PHP upgrades", "Free performance — stay on a supported version, automate with Rector."],
            ["OPcache", "On in prod, timestamps off, clear it on every deploy."],
            ["Env parity", "Differences live in config, never code — same artifact, different .env."],
            ["Observability", "Logs, metrics and traces — make production tell you what's wrong."],
        ],
    ),
    section([
        h(3, "You've got this, Chris"),
        md("The fact you can build production systems, ship real products, and explain "
           "your thinking clearly puts you ahead of most candidates. Read the *Why it "
           "matters* boxes once more, then go in and have a conversation — not an exam."),
    ], bg=INK, tc=INK_T, pad="22pt"),
]))

# ---- assemble --------------------------------------------------------------
doc = {
    "frontmatter": {
        "title": "BaseKit Interview Prep Guide",
        "subtitle": "PHP Backend Developer — Round 2",
        "author": "Chris Garlick",
        "recipient": "Interview Preparation",
        "date": "2026-06-11",
        "document_type": "brief",
        "client": "chris-garlick-light",
    },
    "pages": pages,
}

with open("php-basekit.json", "w") as f:
    json.dump(doc, f, indent=2, ensure_ascii=False)

print(f"Wrote php-basekit.json — {len(pages)} content pages")
