# BaseKit Interview Prep Guide
## PHP Backend Developer — Round 2

---

## Table of Contents

1. [PHP 8 Features vs PHP 7](#php-8-features)
2. [Dependency Injection](#dependency-injection)
3. [SOLID Principles](#solid-principles)
4. [MySQL Query Performance & Bottlenecks](#mysql-performance)
5. [REST API Design](#rest-api)
6. [Docker & Containerisation](#docker)
7. [Linux & Mac Command Line Essentials](#linux-mac)
8. [Git Workflow](#git)
9. [OOP Concepts](#oop)
10. [Security Essentials](#security)
11. [General Best Practices](#best-practices)

---

## 1. PHP 8 Features vs PHP 7 {#php-8-features}

### Why it matters
PHP 8 (released Nov 2020) was a major release with significant performance improvements and new language features. PHP 8.1 and 8.2 continued building on this. Coming from WordPress/PHP 7, these are the things you need to know cold.

---

### Named Arguments (PHP 8.0)
Previously you had to pass arguments in order. Named arguments let you pass them by name, in any order, and skip optional ones.

```php
// PHP 7 — positional only
array_slice($array, 0, 5, true);

// PHP 8 — named, readable, order doesn't matter
array_slice(array: $array, offset: 0, length: 5, preserve_keys: true);
```

Great for built-in functions with many optional parameters. Also works with your own functions.

---

### Union Types (PHP 8.0)
You can now declare that a parameter or return value can be one of multiple types.

```php
// PHP 7 — no way to express this in the signature
function processInput($input) { ... }

// PHP 8
function processInput(int|string $input): int|string {
    return $input;
}
```

---

### Nullsafe Operator `?->` (PHP 8.0)
Eliminates deeply nested null checks. If any part of the chain is null, the whole expression returns null instead of throwing an error.

```php
// PHP 7 — verbose null checking
$country = null;
if ($user !== null) {
    if ($user->getAddress() !== null) {
        $country = $user->getAddress()->getCountry();
    }
}

// PHP 8 — clean chain
$country = $user?->getAddress()?->getCountry();
```

---

### Match Expressions (PHP 8.0)
Like `switch` but stricter, cleaner, and returns a value. Uses strict comparison (`===`) unlike switch which uses loose (`==`).

```php
// PHP 7 switch — loose comparison, fall-through risk, verbose
switch ($status) {
    case 1:
        $label = 'Active';
        break;
    case 2:
        $label = 'Inactive';
        break;
    default:
        $label = 'Unknown';
}

// PHP 8 match — returns value, strict, no fall-through
$label = match($status) {
    1 => 'Active',
    2 => 'Inactive',
    default => 'Unknown',
};
```

A `match` with no matching arm and no default throws an `UnhandledMatchError`, which is safer than switch silently doing nothing.

---

### Constructor Property Promotion (PHP 8.0)
Reduces boilerplate when defining and assigning class properties.

```php
// PHP 7 — declare property, declare parameter, assign in constructor
class User {
    private string $name;
    private string $email;

    public function __construct(string $name, string $email) {
        $this->name = $name;
        $this->email = $email;
    }
}

// PHP 8 — all in one
class User {
    public function __construct(
        private string $name,
        private string $email,
    ) {}
}
```

Much cleaner for value objects and DTOs.

---

### Readonly Properties (PHP 8.1)
A property that can only be set once — at initialisation time. After that, it's immutable.

```php
class Order {
    public function __construct(
        public readonly int $id,
        public readonly string $status,
    ) {}
}

$order = new Order(1, 'pending');
echo $order->id; // 1
$order->id = 2;  // Error: Cannot modify readonly property
```

Perfect for immutable value objects.

---

### Enums (PHP 8.1)
Native enumerations — no more class constants or magic strings.

```php
// Before — fragile, no type safety
const STATUS_ACTIVE = 'active';
const STATUS_INACTIVE = 'inactive';

// PHP 8.1 Enum
enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
}

// Usage
function setStatus(Status $status): void { ... }
setStatus(Status::Active);

// Backed enums can cast from value
$status = Status::from('active'); // Status::Active
```

---

### Fibers (PHP 8.1)
Lightweight concurrency — similar to coroutines. Allows pausing and resuming execution. Important for async-style programming without needing a full event loop like ReactPHP.

```php
$fiber = new Fiber(function(): void {
    $value = Fiber::suspend('first');
    echo "Got: $value\n";
});

$result = $fiber->start();    // runs until suspend, returns 'first'
$fiber->resume('hello');       // resumes, prints "Got: hello"
```

You likely won't be asked to write Fiber code but knowing what they are and why they exist is enough.

---

### First-Class Callable Syntax (PHP 8.1)
Cleaner way to pass functions as callables.

```php
// Before
$fn = Closure::fromCallable('strlen');
$fn = function(string $s) { return strlen($s); };

// PHP 8.1
$fn = strlen(...);
array_map(strlen(...), $strings);
```

---

### Intersection Types (PHP 8.1)
Where union types say "this OR that", intersection types say "this AND that" — the value must implement all of them.

```php
function process(Iterator&Countable $collection): void { ... }
```

---

### readonly Classes (PHP 8.2)
Makes ALL properties readonly by default.

```php
readonly class Point {
    public function __construct(
        public float $x,
        public float $y,
    ) {}
}
```

---

## 2. Dependency Injection {#dependency-injection}

### What it is (plain English)
Instead of a class creating its own dependencies, you *pass them in from outside*. The class declares what it needs; something else (a container, your bootstrap code, a framework) provides it.

Think of it like a restaurant kitchen. The chef doesn't go to the farm to get ingredients — someone delivers them. The chef just needs to know what to cook with.

---

### The problem it solves

```php
// WITHOUT dependency injection — tightly coupled, hard to test
class OrderService {
    public function __construct() {
        $this->db = new MySQLDatabase(); // hard dependency
        $this->mailer = new SmtpMailer(); // hard dependency
    }
}

// You can't test OrderService without a real database and real SMTP
```

```php
// WITH dependency injection — loosely coupled, testable
class OrderService {
    public function __construct(
        private DatabaseInterface $db,
        private MailerInterface $mailer,
    ) {}
}

// Now in tests you can pass in mock/fake implementations
$service = new OrderService(new FakeDatabase(), new FakeMailer());
```

---

### Three types of DI

**Constructor injection** — most common, dependencies required upfront
```php
public function __construct(private LoggerInterface $logger) {}
```

**Setter injection** — optional dependencies set after construction
```php
public function setLogger(LoggerInterface $logger): void {
    $this->logger = $logger;
}
```

**Interface injection** — the dependency provides an injector method (rare)

---

### DI Containers
In real apps, you use a container (like Laravel's service container, Symfony's DI component, or PHP-DI) to wire everything together automatically.

```php
// Binding an interface to a concrete implementation
$container->bind(DatabaseInterface::class, MySQLDatabase::class);

// Now whenever OrderService is resolved, it gets a MySQLDatabase
$service = $container->make(OrderService::class);
```

---

### Why it matters at BaseKit
Their job ads explicitly mention dependency injection as a core practice. They also use it heavily on the frontend (React) and likely throughout their PHP services. Being able to articulate *why* DI is good (testability, loose coupling, single responsibility) is as important as knowing how to use it.

---

## 3. SOLID Principles {#solid-principles}

BaseKit mention SOLID explicitly in job ads. Know them, know a PHP example for each.

---

### S — Single Responsibility Principle
A class should have one, and only one, reason to change.

```php
// BAD — this class does too much
class User {
    public function save(): void { /* DB logic */ }
    public function sendWelcomeEmail(): void { /* Email logic */ }
    public function generateReport(): void { /* Report logic */ }
}

// GOOD — each concern is separate
class UserRepository { public function save(User $user): void {} }
class UserMailer { public function sendWelcome(User $user): void {} }
class UserReporter { public function generate(User $user): void {} }
```

---

### O — Open/Closed Principle
Open for extension, closed for modification. Add new behaviour without changing existing code.

```php
// BAD — you have to modify this class every time a new payment type is added
class PaymentProcessor {
    public function process(string $type, float $amount): void {
        if ($type === 'stripe') { ... }
        elseif ($type === 'paypal') { ... }
        // keep adding elseifs forever...
    }
}

// GOOD — new payment types extend without touching existing code
interface PaymentGateway {
    public function charge(float $amount): void;
}

class StripeGateway implements PaymentGateway {
    public function charge(float $amount): void { ... }
}

class PaypalGateway implements PaymentGateway {
    public function charge(float $amount): void { ... }
}

class PaymentProcessor {
    public function process(PaymentGateway $gateway, float $amount): void {
        $gateway->charge($amount);
    }
}
```

---

### L — Liskov Substitution Principle
Subtypes must be substitutable for their base types without breaking the program.

```php
// BAD — Square breaks the contract of Rectangle
class Rectangle {
    public function setWidth(int $w): void { $this->width = $w; }
    public function setHeight(int $h): void { $this->height = $h; }
}

class Square extends Rectangle {
    // Square forces both dimensions equal, breaking Rectangle's contract
    public function setWidth(int $w): void {
        $this->width = $w;
        $this->height = $w; // side effect! breaks LSP
    }
}

// GOOD — use a shared interface instead of inheritance when behaviour differs
interface Shape {
    public function area(): float;
}
```

---

### I — Interface Segregation Principle
Don't force classes to implement methods they don't use. Many small interfaces beat one fat one.

```php
// BAD — not all workers can eat lunch or take breaks
interface Worker {
    public function work(): void;
    public function eatLunch(): void;
    public function takeBreak(): void;
}

// GOOD — split into focused interfaces
interface Workable { public function work(): void; }
interface Breakable { public function takeBreak(): void; }

class HumanWorker implements Workable, Breakable { ... }
class RobotWorker implements Workable { ... } // robots don't need breaks
```

---

### D — Dependency Inversion Principle
High-level modules shouldn't depend on low-level modules. Both should depend on abstractions.

```php
// BAD — OrderService directly depends on a concrete class
class OrderService {
    private MySQLDatabase $db; // concrete low-level module
}

// GOOD — both depend on an abstraction
interface DatabaseInterface {
    public function query(string $sql): array;
}

class OrderService {
    public function __construct(private DatabaseInterface $db) {}
}

class MySQLDatabase implements DatabaseInterface { ... }
class SQLiteDatabase implements DatabaseInterface { ... }
```

---

## 4. MySQL Query Performance & Bottlenecks {#mysql-performance}

This came up in your first interview, expect more in round 2.

---

### EXPLAIN
The most important tool. Run it before any query you're optimising.

```sql
EXPLAIN SELECT * FROM orders WHERE user_id = 42;
```

Key columns to understand:
- **type** — how MySQL scans the table. Best to worst: `const` → `ref` → `range` → `index` → `ALL`. `ALL` means full table scan — bad on large tables
- **key** — which index is being used (NULL = no index used)
- **rows** — estimated rows MySQL will examine
- **Extra** — look for `Using filesort` or `Using temporary` — these are expensive operations

```sql
-- Even better in MySQL 8+
EXPLAIN ANALYZE SELECT * FROM orders WHERE user_id = 42;
-- Shows actual execution time, not just estimates
```

---

### Indexes
An index is a separate data structure that lets MySQL find rows without scanning the whole table. Like an index in a book.

```sql
-- Check existing indexes
SHOW INDEX FROM orders;

-- Add an index
CREATE INDEX idx_user_id ON orders(user_id);

-- Composite index — order matters, leftmost prefix rule applies
CREATE INDEX idx_user_status ON orders(user_id, status);
-- This index helps: WHERE user_id = 1 AND status = 'active'
-- This index helps: WHERE user_id = 1
-- This index does NOT help: WHERE status = 'active' (no leftmost prefix)
```

**When indexes hurt**: Too many indexes slow down writes (INSERT/UPDATE/DELETE) because MySQL has to maintain each index. Don't index everything blindly.

---

### The N+1 Problem
One of the most common performance killers, especially in ORMs.

```php
// N+1 — 1 query to get orders, then 1 query PER order to get the user
$orders = Order::all(); // 1 query

foreach ($orders as $order) {
    echo $order->user->name; // 1 query per iteration = N queries
}
// With 500 orders = 501 queries

// FIXED — eager load the relationship
$orders = Order::with('user')->get(); // 2 queries total
```

In raw SQL, use JOINs:
```sql
-- Instead of separate queries, join them
SELECT orders.*, users.name
FROM orders
JOIN users ON orders.user_id = users.id;
```

---

### Slow Query Log
MySQL can log queries that take longer than a threshold.

```sql
-- Check if slow query log is on
SHOW VARIABLES LIKE 'slow_query_log%';

-- Enable it
SET GLOBAL slow_query_log = 'ON';
SET GLOBAL long_query_time = 1; -- log queries over 1 second
```

On Linux/Mac the log file is usually at `/var/log/mysql/mysql-slow.log`. Use `mysqldumpslow` to summarise it:

```bash
mysqldumpslow -s t -t 10 /var/log/mysql/mysql-slow.log
# -s t = sort by time, -t 10 = top 10
```

---

### Other Common Bottlenecks

**SELECT *** — fetches all columns even ones you don't need. Always select only what you need:
```sql
-- BAD
SELECT * FROM users WHERE id = 1;

-- GOOD
SELECT id, name, email FROM users WHERE id = 1;
```

**Missing WHERE clause on large tables** — always filter early.

**LIKE with leading wildcard** — can't use an index:
```sql
-- BAD — full table scan
SELECT * FROM products WHERE name LIKE '%widget%';

-- Consider FULLTEXT indexes for search
ALTER TABLE products ADD FULLTEXT(name);
SELECT * FROM products WHERE MATCH(name) AGAINST('widget');
```

**Transactions** — wrap multiple related writes in a transaction to ensure consistency and improve performance:
```sql
START TRANSACTION;
INSERT INTO orders (user_id, total) VALUES (1, 99.99);
INSERT INTO order_items (order_id, product_id) VALUES (LAST_INSERT_ID(), 5);
COMMIT;
```

---

## 5. REST API Design {#rest-api}

### Core principles
- Resources are nouns, not verbs
- Use HTTP methods to express intent
- Stateless — each request contains everything needed

```
GET    /users          — list users
GET    /users/42       — get user 42
POST   /users          — create user
PUT    /users/42       — replace user 42
PATCH  /users/42       — partially update user 42
DELETE /users/42       — delete user 42
```

### HTTP Status Codes to know
```
200 OK              — success
201 Created         — resource created (POST)
204 No Content      — success, nothing to return (DELETE)
400 Bad Request     — client sent invalid data
401 Unauthorized    — not authenticated
403 Forbidden       — authenticated but not allowed
404 Not Found       — resource doesn't exist
422 Unprocessable   — validation failed
429 Too Many Req    — rate limited
500 Server Error    — something broke on your end
```

### Versioning
```
/api/v1/users
/api/v2/users
```
Always version your API so you can introduce breaking changes without breaking existing clients.

### PHP Example (clean REST endpoint)
```php
// routes/api.php (Laravel style)
Route::get('/users/{id}', [UserController::class, 'show']);

// UserController.php
public function show(int $id): JsonResponse
{
    $user = $this->userRepository->findById($id);

    if (!$user) {
        return response()->json(['error' => 'User not found'], 404);
    }

    return response()->json($user, 200);
}
```

---

## 6. Docker & Containerisation {#docker}

BaseKit use Docker and Kubernetes. You don't need to be a DevOps expert but know the basics.

### What Docker does
Packages your app and all its dependencies into a container — an isolated, reproducible environment. "Works on my machine" becomes a solved problem.

### Key concepts
- **Image** — a blueprint (like a class)
- **Container** — a running instance of an image (like an object)
- **Dockerfile** — instructions to build an image
- **docker-compose** — run multiple containers together (app + database + cache)

### Essential commands (Mac/Linux)
```bash
# Pull an image
docker pull php:8.2-fpm

# Run a container
docker run -d -p 8080:80 --name myapp nginx

# List running containers
docker ps

# List all containers (including stopped)
docker ps -a

# Stop a container
docker stop myapp

# Remove a container
docker rm myapp

# View logs
docker logs myapp
docker logs -f myapp  # follow/tail logs

# Execute a command inside a running container
docker exec -it myapp bash
docker exec -it myapp php artisan migrate

# Build from Dockerfile
docker build -t myapp:latest .
```

### Typical Dockerfile for PHP
```dockerfile
FROM php:8.2-fpm

# Install extensions
RUN docker-php-ext-install pdo pdo_mysql

# Set working directory
WORKDIR /var/www

# Copy files
COPY . .

# Install Composer dependencies
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer
RUN composer install --no-dev --optimize-autoloader

EXPOSE 9000
CMD ["php-fpm"]
```

### docker-compose example
```yaml
version: '3.8'
services:
  app:
    build: .
    ports:
      - "8000:8000"
    depends_on:
      - db
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
  db_data:
```

```bash
# Start all services
docker-compose up -d

# Stop all services
docker-compose down

# View logs for all services
docker-compose logs -f
```

### Kubernetes (high level)
Kubernetes (k8s) orchestrates containers at scale. You probably won't be asked to write k8s config but know what it does:
- **Pod** — smallest unit, one or more containers
- **Deployment** — manages rolling updates and replicas
- **Service** — exposes pods to network traffic
- **Ingress** — routes external traffic to services

---

## 7. Linux & Mac Command Line Essentials {#linux-mac}

Both Mac and Linux use bash/zsh. These are the commands that come up day-to-day as a backend dev.

### Navigation
```bash
pwd                    # where am I?
ls -la                 # list all files including hidden, with details
cd /var/www/html       # change directory
cd ..                  # go up one level
cd ~                   # go to home directory
```

### File operations
```bash
cat file.txt           # print file contents
less file.txt          # paginated view (q to quit)
tail -f logfile.log    # follow a log file in real time
head -n 20 file.txt    # first 20 lines
grep "error" log.txt   # search for "error" in file
grep -r "findme" ./    # recursive search in directory
find . -name "*.php"   # find all PHP files
```

### Process management
```bash
ps aux                 # list all running processes
ps aux | grep php      # find PHP processes
kill 1234              # kill process by PID
kill -9 1234           # force kill
top                    # live process monitor
htop                   # better version of top (if installed)
```

### Permissions
```bash
ls -la                 # see permissions (e.g. -rwxr-xr-x)
chmod 755 script.sh    # owner can rwx, others can rx
chmod +x script.sh     # make executable
chown www-data:www-data /var/www  # change owner
sudo command           # run as superuser
```

### Networking
```bash
curl -I https://example.com          # get response headers
curl -X POST -d '{"key":"val"}' \
  -H "Content-Type: application/json" \
  https://api.example.com/endpoint   # POST request

wget https://example.com/file.zip    # download a file
ping google.com                      # test connectivity
netstat -tulpn                       # see what's listening on which ports
lsof -i :8080                        # what's using port 8080
```

### SSH
```bash
ssh user@192.168.1.1          # connect to server
ssh -i ~/.ssh/mykey user@host # connect with specific key
scp file.txt user@host:/path  # copy file to server
```

### Useful shortcuts
```bash
Ctrl+C    # kill running process
Ctrl+Z    # suspend process (bg to resume in background)
Ctrl+R    # search command history
!!        # repeat last command
!$        # last argument of previous command
history   # show command history
```

### PHP specific
```bash
php -v                          # PHP version
php -m                          # loaded modules
php -r "echo phpversion();"     # run inline PHP
php artisan migrate             # Laravel migrations
composer install                # install dependencies
composer update                 # update dependencies
composer dump-autoload          # regenerate autoload files
```

---

## 8. Git Workflow {#git}

### Everyday commands
```bash
git status                      # what's changed
git diff                        # see unstaged changes
git add .                       # stage all changes
git add src/file.php            # stage specific file
git commit -m "message"         # commit
git push origin feature/my-branch

git pull origin main            # pull latest
git fetch --all                 # fetch all remotes without merging

git log --oneline               # compact commit history
git log --oneline --graph       # visual branch history
```

### Branching
```bash
git checkout -b feature/new-thing    # create and switch to branch
git checkout main                    # switch branch
git merge feature/new-thing          # merge into current branch
git rebase main                      # rebase onto main (cleaner history)
git branch -d feature/new-thing      # delete branch after merge
```

### Fixing mistakes
```bash
git stash                       # temporarily shelve changes
git stash pop                   # bring them back
git reset HEAD~1                # undo last commit, keep changes staged
git reset --hard HEAD~1         # undo last commit, DISCARD changes
git revert abc1234              # create new commit that undoes a commit (safe for shared branches)
```

### Good commit message format
```
feat: add user authentication endpoint
fix: resolve null pointer in order processor
refactor: extract payment logic into service class
chore: update composer dependencies
```

---

## 9. OOP Concepts {#oop}

### The four pillars

**Encapsulation** — hide internal state, expose only what's needed
```php
class BankAccount {
    private float $balance = 0;

    public function deposit(float $amount): void {
        if ($amount <= 0) throw new InvalidArgumentException();
        $this->balance += $amount;
    }

    public function getBalance(): float {
        return $this->balance;
    }
    // balance is protected — can't be set directly from outside
}
```

**Abstraction** — expose what something does, not how it does it
```php
interface Cache {
    public function get(string $key): mixed;
    public function set(string $key, mixed $value, int $ttl): void;
}

// Redis, Memcached, file cache — all implement the same interface
// Code using Cache doesn't need to know which one it is
```

**Inheritance** — child classes extend parent behaviour
```php
abstract class Notification {
    abstract public function send(string $message): void;

    public function log(string $message): void {
        echo "[LOG] Sending: $message\n";
    }
}

class EmailNotification extends Notification {
    public function send(string $message): void {
        $this->log($message);
        // send via SMTP
    }
}
```

**Polymorphism** — different classes respond to the same interface differently
```php
$notifications = [new EmailNotification(), new SmsNotification(), new PushNotification()];

foreach ($notifications as $notification) {
    $notification->send("Hello!"); // each does it differently, call is the same
}
```

---

### Composition over Inheritance
BaseKit explicitly mention preferring this. Instead of inheriting behaviour, inject it.

```php
// Inheritance approach — fragile, rigid hierarchy
class LoggingOrderService extends OrderService {
    public function create(): void {
        $this->log("Creating order");
        parent::create();
    }
}

// Composition approach — flexible, testable
class OrderService {
    public function __construct(
        private OrderRepository $repo,
        private LoggerInterface $logger,  // composed in
    ) {}

    public function create(): void {
        $this->logger->info("Creating order");
        $this->repo->save();
    }
}
```

---

## 10. Security Essentials {#security}

### SQL Injection (since it came up!)
Never interpolate user input into SQL strings.

```php
// VULNERABLE
$id = $_GET['id']; // attacker passes: 1 OR 1=1
$result = $db->query("SELECT * FROM users WHERE id = $id");

// SAFE — prepared statements
$stmt = $db->prepare("SELECT * FROM users WHERE id = ?");
$stmt->execute([$id]);
```

### XSS (Cross-Site Scripting)
Escape output in HTML contexts.
```php
// VULNERABLE
echo "<p>Hello " . $_GET['name'] . "</p>";
// attacker passes: <script>stealCookies()</script>

// SAFE
echo "<p>Hello " . htmlspecialchars($_GET['name'], ENT_QUOTES, 'UTF-8') . "</p>";
```

### CSRF (Cross-Site Request Forgery)
Use tokens to verify form submissions come from your own site. Laravel handles this automatically with `@csrf`.

### Password Hashing
```php
// NEVER store plain text or use MD5/SHA1
// ALWAYS use password_hash
$hash = password_hash($plaintext, PASSWORD_BCRYPT);

// Verify
if (password_verify($plaintext, $hash)) {
    // authenticated
}
```

### Environment Variables
Never commit secrets to git.
```bash
# .env
DB_PASSWORD=supersecret
STRIPE_KEY=sk_live_xxxxx

# .gitignore
.env
```

---

## 11. General Best Practices {#best-practices}

### Code review mindset
When reviewing or being reviewed on code, think about:
- Is this readable in 6 months?
- Is this tested?
- Are there edge cases unhandled?
- Is there a simpler way?

### Testing basics
```php
// Unit test example (PHPUnit)
class OrderServiceTest extends TestCase {
    public function test_creates_order_successfully(): void {
        $mockRepo = $this->createMock(OrderRepository::class);
        $mockRepo->expects($this->once())->method('save');

        $service = new OrderService($mockRepo);
        $service->create(['user_id' => 1, 'total' => 50.00]);
    }
}
```

### Don't Repeat Yourself (DRY)
If you're writing the same logic twice, extract it.

### YAGNI — You Aren't Gonna Need It
Don't build features or abstractions for requirements that don't exist yet. BaseKit move fast — over-engineering slows that down.

### KISS — Keep It Simple
The simplest solution that works is almost always the right one. Complexity should be earned, not assumed.

---

## Quick Reference — Things to Say in the Interview

| Topic | Key phrase |
|---|---|
| DI | "Loose coupling and testability — you program to interfaces not implementations" |
| SOLID | "Each principle helps keep classes small, focused and easy to change" |
| N+1 | "Eager load relationships rather than lazy loading in a loop" |
| Slow queries | "First thing I'd do is run EXPLAIN and check if we're doing a full table scan" |
| Composition | "I prefer composing behaviour through injected dependencies over deep inheritance hierarchies" |
| Testing | "If something is hard to test, it's usually a sign the design needs revisiting" |

---

*Good luck Chris — you've got this. The fact you can build production systems, ship real products and explain your thinking clearly puts you ahead of most candidates.*