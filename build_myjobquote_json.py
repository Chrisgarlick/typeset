#!/usr/bin/env python3
"""Generate php-myjobquote.json — a serious Laravel revision guide.

Target role: Senior Backend Laravel Developer at a high-growth marketplace app
(remote-first, one day/month in Chester). Must-haves called out in the ad:
Laravel, 5+ yrs frameworked languages, MySQL/MariaDB indexes & table design,
big data, many APIs (Stripe / Google / AWS / OpenAI), data caching, job queues,
modernising a Laravel app, 800+ models, microservices, light DevOps.

Uses the Typeset JSON layout schema (heading/markdown/code/table/columns/section).
json.dumps handles all escaping so the output is always valid.
"""
import json

# ---- palette ---------------------------------------------------------------
INK    = "#0F172A"   # slate-900 — "why it matters" / "say this" boxes (light text)
INK_T  = "#F1F5F9"
RED    = "#B91C1C"   # Laravel-red accent — part dividers / role banner
RED_T  = "#FEF2F2"
CARD   = "#EFF6FF"   # soft blue accent card
CARD_B = "#BFDBFE"
CREAM  = "#FAF7F2"
WHITE  = "#FFFFFF"
LINE   = "#E2E8F0"
BAD    = "#991B1B"
BAD_T  = "#FEF2F2"
GOOD   = "#065F46"
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

def say(text):
    return section([md(text)], bg=INK, tc=INK_T)

def page(blocks):
    return {"blocks": blocks}

pages = []

# ===========================================================================
# Page 1 — Contents + how to use
# ===========================================================================
pages.append(page([
    h(2, "About This Guide"),
    md("A focused Laravel refresher for the **Senior Backend Laravel Developer** role — "
       "a fast-growing, remote-first marketplace app used by *millions*, where the job "
       "is to help **modernise their Laravel application**. Each section opens with "
       "*why it matters for this role*, gives the concept in plain English, then a "
       "Laravel/PHP example you can speak to."),
    columns([
        slot([
            h(4, "LARAVEL CORE"),
            md("1. The Role, Mapped\n2. Request Lifecycle\n3. Service Container & DI\n"
               "4. Providers & Facades\n5. Routing & Middleware\n6. Validation\n"
               "7. API Resources"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "ELOQUENT & DATA"),
            md("8. Eloquent Relationships\n9. N+1 & Eager Loading\n10. Scopes, Casts, Observers\n"
               "11. Indexes & Table Design\n12. EXPLAIN & Tuning\n13. Migrations at Scale\n"
               "14. Big Data in Laravel"),
        ], bg=CREAM),
        slot([
            h(4, "SCALE, APIs & OPS"),
            md("15. Transactions & Locking\n16. Caching / Redis\n17. Queues & Horizon\n"
               "18. Events & Listeners\n19. Stripe / AWS / Google / OpenAI\n"
               "20. Webhooks & Idempotency\n21. Microservices\n22. Testing\n"
               "23. Security\n24. DevOps & Backups\n25. Modernising a Legacy App"),
        ], bg=WHITE, border={"width": "1pt", "color": LINE}),
    ], ratios=[1, 1, 1], gutter="6mm"),
    section([
        md("**How to use it:** read every *Why it matters* box, rehearse the *Say this* "
           "lines out loud, and be ready to tie each topic back to **their app** — a "
           "high-traffic marketplace with 800+ models, heavy data, and lots of API "
           "integrations. The last page has questions to ask *them*.")
    ], bg=CREAM),
]))

# ===========================================================================
# 1. The Role, Mapped
# ===========================================================================
pages.append(page([
    h(2, "1 · The Role, Mapped to You"),
    why("Before the tech, get the story straight. They want a **problem-solver**, not a "
        "task-worker; someone who treats the craft as a passion and wants to *grow with "
        "the company*. Map every must-have to a concrete thing you've built — vague "
        "claims lose to specific examples."),
    h(3, "Their must-haves → your evidence"),
    table(
        ["They want", "What to say / show"],
        [
            ["2+ yrs Laravel", "A Laravel app you shipped — models, migrations, queues, the lot."],
            ["5+ yrs frameworked langs", "Total framework experience across PHP/Laravel and others."],
            ["MySQL indexes & table design", "A slow query you fixed with an index; a schema you designed."],
            ["Big data, and enjoys it", "Largest table/dataset you've handled; how you kept it fast."],
            ["Many APIs (Stripe/Google/AWS/OpenAI)", "An integration you built end-to-end, incl. webhooks."],
            ["Caching & job queues", "Work you pushed to a queue; a cache that cut load."],
            ["Problem-solver mindset", "The 'Google Maps 100x' type story — see §25."],
        ],
    ),
    columns([
        slot([
            h(4, "THE TELL"),
            md("They ask *lots of intelligent questions* and want the same back. Curiosity "
               "about **their** app — its scale, its pain points, its 800+ models — reads "
               "as genuine interest, which they explicitly value."),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "SAY THIS"),
            quote("I like owning a problem end to end — understanding the why, not just "
                  "closing the ticket. On a marketplace at your scale, the interesting "
                  "work is keeping it fast and reliable as the data grows."),
        ], bg=INK, tc=INK_T),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# 2. Request Lifecycle
# ===========================================================================
pages.append(page([
    h(2, "2 · The Laravel Request Lifecycle"),
    why("\"What actually happens when a request hits a Laravel app?\" is a classic senior "
        "opener. Knowing the flow shows you understand the framework, not just its "
        "surface API."),
    section([
        h(4, "THE PATH OF A REQUEST"),
        md("`public/index.php` → **bootstrap** the app → the **HTTP Kernel** runs global "
           "**middleware** → the **router** matches a route → route/controller middleware "
           "→ the **controller** action runs → a **Response** flows back out through the "
           "middleware → sent to the client. Service providers are **registered and "
           "booted** during bootstrap, before routing."),
    ], bg=INK, tc=INK_T),
    columns([
        slot([
            h(4, "MIDDLEWARE = LAYERS"),
            md("Like an onion — each request passes *in* through the layers and the "
               "response passes back *out*. Auth, CORS, throttling, and session all live "
               "here. Great mental model to have ready."),
            code("php",
"""public function handle($request, Closure $next)
{
    // before...
    $response = $next($request);
    // after...
    return $response;
}"""),
        ], bg=CREAM),
        slot([
            h(4, "SHARED-NOTHING"),
            md("Each request boots a **fresh** app and tears it down after (unless you run "
               "Octane). So no in-memory state survives between requests — state lives in "
               "**Redis / the database**. This is *why* Laravel scales horizontally: add "
               "more PHP-FPM workers behind a load balancer."),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    md("**Worth naming:** *Laravel Octane* (Swoole/RoadRunner) keeps the app booted in "
       "memory between requests for big throughput gains — but you must avoid leaking "
       "state between requests. Good to mention as a modernisation lever."),
]))

# ===========================================================================
# 3. Service Container & DI
# ===========================================================================
pages.append(page([
    h(2, "3 · Service Container & DI"),
    why("DI is the backbone of a testable Laravel app, and modernising a legacy app "
        "almost always means moving logic *out* of fat controllers and *into* injected, "
        "single-responsibility services. Be fluent here."),
    section([
        md("**Plain English:** the container is Laravel's *factory + registry*. You "
           "declare what a class needs in its constructor; the container builds and "
           "injects those dependencies for you (**automatic resolution**). You program to "
           "**interfaces**, and bind the concrete implementation in one place.")
    ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    columns([
        slot([
            h(4, "BIND AN INTERFACE"),
            code("php",
"""// In a service provider's register()
$this->app->bind(
    PaymentGateway::class,
    StripeGateway::class,
);

// singleton — one shared instance
$this->app->singleton(
    SearchClient::class,
    fn($app) => new SearchClient(config('search.key')),
);"""),
        ], bg=CREAM),
        slot([
            h(4, "IT JUST INJECTS"),
            code("php",
"""class CheckoutController
{
    public function __construct(
        private PaymentGateway $gateway,
    ) {}
    // Laravel resolves StripeGateway
    // automatically — no `new`
}"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
    md("**bind vs singleton:** `bind` resolves a *fresh* instance each time; `singleton` "
       "resolves *once* and reuses it. **Contextual binding** lets two classes get "
       "different implementations of the same interface. **Method injection** works too "
       "— type-hint a dependency in a controller action and Laravel injects it."),
    say("**Say this:** *\"I bind interfaces to implementations in a provider and let the "
        "container inject them — it gives me loose coupling and lets me swap a fake in "
        "tests without touching the class.\"*"),
]))

# ===========================================================================
# 4. Providers & Facades
# ===========================================================================
pages.append(page([
    h(2, "4 · Service Providers & Facades"),
    columns([
        slot([
            h(4, "SERVICE PROVIDERS"),
            md("The **bootstrapping** centre of the app. `register()` binds things into "
               "the container (no resolving other services yet); `boot()` runs after all "
               "providers are registered — the place for event listeners, route model "
               "bindings, view composers, macros."),
            code("php",
"""class AppServiceProvider extends ServiceProvider
{
    public function register(): void
    {
        $this->app->singleton(Pricing::class);
    }
    public function boot(): void
    {
        Model::preventLazyLoading(! app()->isProduction());
    }
}"""),
        ], bg=CREAM),
        slot([
            h(4, "FACADES"),
            md("A **static-looking proxy** to a service resolved from the container — "
               "`Cache::get()` is really `app('cache')->get()` under the hood. Convenient, "
               "and **fully mockable** in tests (`Cache::shouldReceive(...)`)."),
            code("php",
"""Cache::put('k', $v, 600);
DB::transaction(fn() => ...);
Log::info('done');

// The DI purist's alternative:
public function __construct(
    private CacheRepository $cache,
) {}"""),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    md("**Talking point for modernisation:** facades are quick but hide dependencies. In "
       "new code, **constructor injection** is easier to test and reason about; facades "
       "are fine for framework-level concerns (`Log`, `Cache`, `DB`). Knowing *when* to "
       "use which is the senior signal."),
    section([
        h(4, "DEFERRED PROVIDERS"),
        md("A provider can be **deferred** so its bindings load lazily only when needed — "
           "a small boot-time win across a large app with many providers."),
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 5. Routing & Middleware
# ===========================================================================
pages.append(page([
    h(2, "5 · Routing, Middleware & Controllers"),
    columns([
        slot([
            h(4, "ROUTES & MODEL BINDING"),
            code("php",
"""Route::apiResource('orders', OrderController::class);

Route::get('/orders/{order}', function (Order $order) {
    // route-model binding: {order} is
    // auto-resolved by id (404 if missing)
    return new OrderResource($order);
})->middleware('auth:sanctum');"""),
        ], bg=CREAM),
        slot([
            h(4, "MIDDLEWARE USES"),
            md("- **auth / sanctum** — who are you\n"
               "- **throttle** — rate limiting (`throttle:60,1`)\n"
               "- **can** — authorize a policy\n"
               "- Custom: tenant scoping, request-id, audit logging"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    h(3, "Keep controllers thin"),
    md("A fat controller is the #1 legacy smell. Validate in a **Form Request**, do work "
       "in an injected **service** or **action** class, and return an **API Resource**. "
       "The controller just orchestrates."),
    columns([
        slot([
            h(4, "✗ FAT CONTROLLER"),
            code("php",
"""public function store(Request $r)
{
    $data = $r->validate([...]);
    // 60 lines of business logic,
    // DB writes, emails, API calls...
}"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ THIN CONTROLLER"),
            code("php",
"""public function store(
    StoreOrderRequest $request,
    PlaceOrder $placeOrder,
) {
    $order = $placeOrder->handle(
        $request->validated()
    );
    return new OrderResource($order);
}"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# 6. Validation & Form Requests
# ===========================================================================
pages.append(page([
    h(2, "6 · Validation & Form Requests"),
    md("Validation is your first line of defence and a place legacy apps get messy. Move "
       "rules into a **Form Request** so the controller stays clean and the rules are "
       "reusable and testable."),
    code("php",
"""class StoreOrderRequest extends FormRequest
{
    public function authorize(): bool
    {
        return $this->user()->can('create', Order::class);
    }

    public function rules(): array
    {
        return [
            'items'            => ['required', 'array', 'min:1'],
            'items.*.sku'      => ['required', 'string', 'exists:products,sku'],
            'items.*.qty'      => ['required', 'integer', 'min:1'],
            'coupon'           => ['nullable', 'string', new ValidCoupon],
            'shipping_country' => ['required', Rule::in(['GB', 'IE'])],
        ];
    }
}"""),
    columns([
        slot([
            h(4, "WHY IT'S BETTER"),
            md("- `authorize()` gates the request before rules even run\n"
               "- Rules live with the request, not scattered\n"
               "- `$request->validated()` returns only clean data\n"
               "- Custom rules (`ValidCoupon`) are unit-testable"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "WORTH NAMING"),
            md("- **Nested / array rules** with dot notation\n"
               "- `prepareForValidation()` to normalise input\n"
               "- **`Rule::unique()`** ignoring self on update\n"
               "- Validation runs *before* your logic — fail fast"),
        ], bg=CREAM),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# 7. API Resources
# ===========================================================================
pages.append(page([
    h(2, "7 · API Resources & REST in Laravel"),
    why("A marketplace lives and dies by its API. Eloquent API Resources are how you "
        "shape a consistent JSON contract and avoid leaking DB columns straight to the "
        "client."),
    columns([
        slot([
            h(4, "A RESOURCE"),
            code("php",
"""class OrderResource extends JsonResource
{
    public function toArray($request): array
    {
        return [
            'id'     => $this->id,
            'total'  => $this->total_pennies / 100,
            'status' => $this->status,
            'items'  => ItemResource::collection(
                $this->whenLoaded('items')
            ),
        ];
    }
}"""),
        ], bg=CREAM),
        slot([
            h(4, "WHY whenLoaded()"),
            md("Only includes the relationship **if it was eager-loaded** — so the "
               "resource never triggers a hidden N+1 (see §9). This one method is a "
               "senior tell."),
            code("php",
"""// Controller decides what to load
Order::with('items')
     ->paginate(20);
// → OrderResource::collection(...)"""),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    h(3, "REST conventions & versioning"),
    table(
        ["Method / Route", "Meaning", "Status on success"],
        [
            ["GET /api/v1/orders", "list (paginated)", "200"],
            ["POST /api/v1/orders", "create", "201 Created"],
            ["PATCH /api/v1/orders/42", "partial update", "200"],
            ["DELETE /api/v1/orders/42", "delete", "204 No Content"],
            ["(validation fails)", "unprocessable", "422 + error bag"],
            ["(rate limited)", "too many requests", "429"],
        ],
    ),
    md("**Always version** (`/api/v1`) so you can ship breaking changes without breaking "
       "clients — critical for an app used by millions with existing integrations."),
]))

# ===========================================================================
# 8. Eloquent Relationships
# ===========================================================================
pages.append(page([
    h(2, "8 · Eloquent Relationships"),
    why("With 800+ models, relationships *are* the app. Fluency here — and knowing which "
        "relationship maps to which schema shape — is non-negotiable for this role."),
    columns([
        slot([
            h(4, "THE COMMON TYPES"),
            md("- **hasOne / belongsTo** — 1:1, FK on the child\n"
               "- **hasMany** — 1:many\n"
               "- **belongsToMany** — many:many via a **pivot** table\n"
               "- **hasManyThrough** — reach across an intermediate\n"
               "- **morphMany / morphTo** — *polymorphic* (comments on many types)"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "IN CODE"),
            code("php",
"""class User extends Model {
    public function orders() {
        return $this->hasMany(Order::class);
    }
    public function roles() {
        return $this->belongsToMany(Role::class)
            ->withTimestamps()
            ->withPivot('assigned_by');
    }
}"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    h(3, "Polymorphic — the one they may probe"),
    md("A `comments` table with `commentable_id` + `commentable_type` can attach to "
       "orders, products, users — anything. Powerful, but the polymorphic column can't "
       "have a normal foreign key, so integrity is enforced in code."),
    code("php",
"""class Comment extends Model {
    public function commentable() { return $this->morphTo(); }
}
class Product extends Model {
    public function comments() { return $this->morphMany(Comment::class, 'commentable'); }
}"""),
]))

# ===========================================================================
# 9. N+1 & Eager Loading
# ===========================================================================
pages.append(page([
    h(2, "9 · The N+1 Problem"),
    why("This came up in their world for sure — it's the single most common performance "
        "killer in any Eloquent app, and it gets *worse* the more data you have. On a "
        "marketplace, an unnoticed N+1 can fire thousands of queries per page."),
    columns([
        slot([
            h(4, "✗ N+1 — 1 + N QUERIES"),
            code("php",
"""$orders = Order::all();       // 1
foreach ($orders as $order) {
    echo $order->user->name;  // +1 each
}
// 500 orders = 501 queries"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ EAGER LOAD — 2 QUERIES"),
            code("php",
"""$orders = Order::with('user')->get();
foreach ($orders as $order) {
    echo $order->user->name;
}
// 2 queries, regardless of count"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
    h(3, "The tools"),
    md("- **`with()`** — eager load up front. Nested & constrained: "
       "`with(['items.product', 'user:id,name'])`.\n"
       "- **`load()`** — lazy-eager load on an existing model.\n"
       "- **`withCount()`** — counts without loading the rows.\n"
       "- **`Model::preventLazyLoading()`** in dev — throws the moment an N+1 happens, so "
       "you catch it *before* production."),
    code("php",
"""// Only load what you need + a count, no N+1
$orders = Order::with(['items:id,order_id,product_id,qty'])
    ->withCount('items')
    ->where('status', 'paid')
    ->latest()
    ->paginate(20);"""),
    say("**Say this:** *\"I eager load relationships rather than lazy loading in a loop, "
        "and I turn on preventLazyLoading in dev so an N+1 fails loudly instead of "
        "silently firing a thousand queries in prod.\"*"),
]))

# ===========================================================================
# 10. Scopes, Casts, Observers
# ===========================================================================
pages.append(page([
    h(2, "10 · Scopes, Casts & Observers"),
    columns([
        slot([
            h(4, "QUERY SCOPES"),
            md("Reusable query fragments. **Global scopes** apply to every query (e.g. "
               "soft-deletes, tenant scoping) — powerful and dangerous."),
            code("php",
"""// local scope
public function scopePaid($q) {
    return $q->where('status', 'paid');
}
Order::paid()->latest()->get();"""),
        ], bg=CREAM),
        slot([
            h(4, "CASTS"),
            md("Cast raw DB columns to rich types automatically — no manual "
               "encode/decode."),
            code("php",
"""protected function casts(): array {
    return [
        'meta'      => 'array',
        'is_active' => 'boolean',
        'settings'  => AsEncrypted::class,
        'price'     => MoneyCast::class,
    ];
}"""),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    h(3, "Observers & model events"),
    md("Hook into a model's lifecycle (`creating`, `updated`, `deleting`, …) to keep "
       "side-effects out of controllers — audit logs, cache busting, denormalised "
       "counters. Keep them **fast**; push slow work to a queue."),
    code("php",
"""class OrderObserver {
    public function created(Order $order): void {
        Cache::forget("user:{$order->user_id}:orders");
        SendOrderConfirmation::dispatch($order);  // queue the slow bit
    }
}"""),
    section([
        h(4, "⚠ GLOBAL SCOPE TRAP"),
        md("A global tenant scope is great — until a raw query or a `withoutGlobalScopes()` "
           "call quietly bypasses it and leaks one customer's data to another. Know that "
           "trade-off before you lean on them."),
    ], bg=BAD, tc=BAD_T),
]))

# ===========================================================================
# Part Two divider — Data at Scale
# ===========================================================================
pages.append(page([
    h(2, "Part Two · Data at Scale"),
    section([
        md("The ad is emphatic about **SQL / MySQL / MariaDB indexes & table design** and "
           "**big data**. This is the half of the interview where a marketplace round "
           "goes deep — because at their scale, the database *is* the bottleneck. Own "
           "this part and you own the interview."),
    ], bg=RED, tc=RED_T, pad="22pt"),
    columns([
        slot([
            h(4, "SCHEMA & INDEXES"),
            md("11. Indexes & Table Design\n12. EXPLAIN & Query Tuning\n"
               "13. Migrations at Scale"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "VOLUME"),
            md("14. Big Data in Laravel\n15. Transactions & Locking\n"
               "16. Caching & Redis"),
        ], bg=CREAM),
        slot([
            h(4, "ASYNC & INTEGRATIONS"),
            md("17. Queues & Horizon\n18. Events & Listeners\n"
               "19. Third-party APIs\n20. Webhooks"),
        ], bg=WHITE, border={"width": "1pt", "color": LINE}),
    ], ratios=[1, 1, 1], gutter="6mm"),
]))

# ===========================================================================
# 11. Indexes & Table Design
# ===========================================================================
pages.append(page([
    h(2, "11 · MySQL/MariaDB Indexes & Design"),
    why("They list this as a **must-have**, twice over. Be ready to explain how an index "
        "works, when it *won't* be used, and how you'd design a table that stays fast at "
        "millions of rows."),
    h(3, "How an index actually works"),
    md("InnoDB stores the table itself as a **B-tree keyed on the primary key** — the "
       "*clustered index*. A **secondary index** is a second B-tree whose leaves hold the "
       "PK, so a lookup by a secondary index is two hops: find the PK, then fetch the "
       "row. This is why a small PK (`BIGINT`, not a random UUID) matters — it sits inside "
       "*every* secondary index."),
    columns([
        slot([
            h(4, "COMPOSITE & LEFTMOST-PREFIX"),
            code("sql",
"""CREATE INDEX idx_user_status
  ON orders (user_id, status);
-- helps:  WHERE user_id=1 AND status='paid'
-- helps:  WHERE user_id=1
-- NO help: WHERE status='paid'
--          (no leftmost prefix)"""),
        ], bg=CREAM),
        slot([
            h(4, "COVERING INDEX"),
            md("If the index holds *every* column the query reads, MySQL answers from the "
               "index alone — no row fetch. EXPLAIN shows `Using index`."),
            code("sql",
"""CREATE INDEX idx_cover
  ON orders (user_id, status, total);
-- SELECT total ... WHERE user_id=1
--   AND status='paid'  → index only"""),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    h(3, "Column order: equality, then range, then sort"),
    code("sql",
"""-- WHERE tenant_id=? AND status=? AND created_at > ? ORDER BY created_at
CREATE INDEX idx ON orders (tenant_id, status, created_at);
--                          equality   equality  range + sort
-- Satisfies filter AND sort — no 'Using filesort'."""),
    md("**Design principles:** right-size types (`INT` over `BIGINT` where it fits, "
       "`VARCHAR(50)` over `(255)`, `TIMESTAMP` over `DATETIME`); **normalise** for "
       "integrity, **denormalise a counter** for read speed; never index blindly — every "
       "index slows writes."),
]))

pages.append(page([
    h(2, "11 · When an Index Won't Be Used"),
    md("Knowing these saves hours of head-scratching — and naming them cold is a strong "
       "senior signal."),
    code("sql",
"""-- ✗ function on the column → index ignored
WHERE YEAR(created_at) = 2024;
-- ✓ rewrite as a range so the index works
WHERE created_at >= '2024-01-01' AND created_at < '2025-01-01';

-- ✗ leading wildcard → no index (use FULLTEXT for search)
WHERE name LIKE '%smith';

-- ✗ type mismatch → silent cast, index skipped
WHERE phone = 07911123456;   -- phone is VARCHAR; quote it

-- ✗ low selectivity → optimiser prefers a scan
WHERE is_active = 1;          -- if 95% of rows are active"""),
    columns([
        slot([
            h(4, "FULLTEXT & SEARCH"),
            md("`LIKE '%x%'` can't use a B-tree index. For real search, a **FULLTEXT** "
               "index, or an external engine (Meilisearch / Elasticsearch via **Laravel "
               "Scout**) on a marketplace catalogue."),
        ], bg=CREAM),
        slot([
            h(4, "SELECTIVITY"),
            md("An index only helps if it narrows results a lot. A boolean flag where 95% "
               "of rows match isn't worth indexing on its own — but it can be a useful "
               "*trailing* column in a composite."),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    say("**Say this:** *\"Before adding an index I run EXPLAIN, check selectivity, and "
        "order composite columns equality-first. I reach for a covering index or a "
        "denormalised count before I reach for more hardware.\"*"),
]))

# ===========================================================================
# 12. EXPLAIN & Query Tuning
# ===========================================================================
pages.append(page([
    h(2, "12 · EXPLAIN & Query Tuning"),
    md("EXPLAIN is the **first move** on any slow query — run it before you optimise "
       "anything."),
    code("sql",
"""EXPLAIN SELECT * FROM orders WHERE user_id = 42;
-- MySQL 8+ / MariaDB: actual timings, not estimates
EXPLAIN ANALYZE SELECT * FROM orders WHERE user_id = 42;"""),
    h(3, "Reading it like a senior"),
    table(
        ["Column", "Want to see", "Red flag"],
        [
            ["type", "const, eq_ref, ref, range", "ALL (full table scan)"],
            ["key", "the index you expect", "NULL (no index used)"],
            ["rows", "small vs table size", "millions"],
            ["filtered", "high %", "low % = scan then discard"],
            ["Extra", "Using index", "Using filesort / Using temporary"],
        ],
    ),
    columns([
        slot([
            h(4, "FIND THE OFFENDERS"),
            code("sql",
"""SET GLOBAL slow_query_log = 'ON';
SET GLOBAL long_query_time = 1;
-- summarise, top 10 by time
-- mysqldumpslow -s t -t 10 slow.log"""),
        ], bg=CREAM),
        slot([
            h(4, "IN LARAVEL, IN DEV"),
            code("php",
"""DB::enableQueryLog();
// ...run the page...
logger()->info('q', [
  'n' => count(DB::getQueryLog())
]);
// Debugbar / Clockwork show
// query count + timings per request"""),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    say("**Say this:** *\"First thing I'd do is run EXPLAIN and check whether we're doing "
        "a full table scan or a filesort — nine times out of ten the fix is an index or "
        "killing an N+1, not more hardware.\"*"),
]))

# ===========================================================================
# 13. Migrations at Scale
# ===========================================================================
pages.append(page([
    h(2, "13 · Migrations & Schema at Scale"),
    why("Their app has **800+ models**. Schema change is a daily reality, and on a big, "
        "live table a careless migration is an *outage*. Show you think about migrations "
        "as production operations, not just local `php artisan migrate`."),
    columns([
        slot([
            h(4, "A CLEAN MIGRATION"),
            code("php",
"""Schema::create('orders', function (Blueprint $t) {
    $t->id();                       // BIGINT PK
    $t->foreignId('user_id')
      ->constrained()
      ->cascadeOnDelete();
    $t->string('status')->index();
    $t->unsignedInteger('total_pennies');
    $t->timestamps();
    $t->index(['user_id', 'status']); // composite
});"""),
        ], bg=CREAM),
        slot([
            h(4, "THE BIG-TABLE TRAP"),
            md("Adding a column or index to a multi-million-row table can **lock it** and "
               "stall the app. Mitigations to name:\n\n"
               "- **Online DDL** (`ALGORITHM=INPLACE`) where supported\n"
               "- **pt-online-schema-change** / **gh-ost** for zero-downtime\n"
               "- Deploy schema change and code change in **separate, backward-compatible "
               "steps** (expand → migrate → contract)"),
        ], bg=BAD, tc=BAD_T),
    ], ratios=[1, 1]),
    h(3, "Practical rules"),
    md("- **Every migration is reversible** — a working `down()` matters when a deploy "
       "goes wrong.\n"
       "- Foreign keys enforce integrity; **cascade** deliberately.\n"
       "- Add a column **nullable / with a default** first, backfill in a queued job, "
       "*then* make it required — never one giant blocking migration.\n"
       "- Keep the **schema dump** (`schema:dump`) current so fresh installs don't replay "
       "hundreds of migrations."),
]))

# ===========================================================================
# 14. Big Data in Laravel
# ===========================================================================
pages.append(page([
    h(2, "14 · Big Data in Laravel"),
    why("\"Worked with big data and enjoys it\" is a stated must-have. The core skill is "
        "**never loading more into memory than you need** — and Laravel gives you clean "
        "tools for exactly that."),
    columns([
        slot([
            h(4, "✗ LOADS EVERYTHING"),
            code("php",
"""// pulls every row into memory,
// then hits memory_limit and dies
foreach (Order::all() as $o) {
    export($o);
}"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ STREAM IT"),
            code("php",
"""// N rows at a time, constant memory
Order::chunkById(1000, function ($chunk) {
    foreach ($chunk as $o) export($o);
});

// or a lazy cursor (generator underneath)
foreach (Order::lazy() as $o) export($o);"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
    h(3, "The toolbox"),
    md("- **`chunkById()`** over `chunk()` when you're also *modifying* rows (stable "
       "paging by PK, no skipped rows).\n"
       "- **`cursor()` / `lazy()`** — a `LazyCollection` streaming one row at a time.\n"
       "- **`LazyCollection`** — apply `map`/`filter` lazily over a huge source without "
       "materialising it.\n"
       "- **Bulk insert / upsert** — `Model::upsert($rows, ['sku'], ['price'])` beats "
       "thousands of individual inserts.\n"
       "- **Queue the whole job** (§17) and **batch** it so it runs off the request cycle.\n"
       "- Always **paginate** list endpoints — `LIMIT/OFFSET`, or **keyset** (`WHERE id > "
       "?`) for deep pages where OFFSET gets slow."),
    code("php",
"""// Keyset pagination — scales where OFFSET doesn't
$page = Order::where('id', '>', $lastId)
    ->orderBy('id')
    ->limit(100)
    ->get();"""),
]))

# ===========================================================================
# 15. Transactions & Locking
# ===========================================================================
pages.append(page([
    h(2, "15 · Transactions, ACID & Locking"),
    why("Money and inventory on a marketplace must **not half-happen**. Being able to "
        "describe a race condition *and* its fix is exactly the backend rigour they're "
        "hiring for."),
    columns([
        slot([
            h(4, "ACID + SAFE WRITE"),
            code("php",
"""DB::transaction(function () use ($from, $to) {
    $from->decrement('balance', 100);
    $to->increment('balance', 100);
    // any throw → automatic rollback
});"""),
            md("**A**tomic · **C**onsistent · **I**solated · **D**urable. `DB::transaction()` "
               "commits on success, rolls back on any exception, and can auto-retry on "
               "deadlock."),
        ], bg=CREAM),
        slot([
            h(4, "ISOLATION LEVELS"),
            md("- READ COMMITTED — non-repeatable reads (Postgres default)\n"
               "- **REPEATABLE READ** — MySQL/InnoDB default\n"
               "- SERIALIZABLE — safest, slowest\n\n"
               "InnoDB's **gap locking** largely prevents phantoms even at REPEATABLE READ."),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    h(3, "The classic race: last item in stock"),
    columns([
        slot([
            h(4, "✗ LOST UPDATE"),
            code("php",
"""$stock = $product->stock;   // both read 1
if ($stock > 0) {           // both pass
    $product->decrement('stock');
}
// sold twice, stock = -1"""),
        ], bg=BAD, tc=BAD_T),
        slot([
            h(4, "✓ LOCK THE ROW"),
            code("php",
"""DB::transaction(function () use ($id) {
    $p = Product::whereKey($id)
        ->lockForUpdate()   // 2nd req waits
        ->first();
    if ($p->stock > 0) $p->decrement('stock');
});"""),
        ], bg=GOOD, tc=GOOD_T),
    ], ratios=[1, 1]),
    say("**Even simpler:** push the check into the write — `UPDATE products SET stock = "
        "stock - 1 WHERE id = ? AND stock > 0` — then check affected rows. The database "
        "does the locking and stock can never go negative."),
]))

# ===========================================================================
# 16. Caching & Redis
# ===========================================================================
pages.append(page([
    h(2, "16 · Caching Strategies & Redis"),
    why("Data caching is a stated must-have. On a read-heavy marketplace, the right cache "
        "is the difference between the DB coping and the DB falling over."),
    columns([
        slot([
            h(4, "THE LAYERS"),
            md("- **OPcache** — compiled PHP bytecode (on in prod, always)\n"
               "- **App cache** — Redis for data/results\n"
               "- **HTTP cache** — `Cache-Control`, ETags, CDN for assets\n"
               "- **Query/computed results** — expensive aggregates"),
        ], bg=INK, tc=INK_T),
        slot([
            h(4, "CACHE-ASIDE (LARAVEL)"),
            code("php",
"""$user = Cache::remember(
    "user:$id", 3600,
    fn() => User::findOrFail($id),
);
// bust on write:
Cache::forget("user:$id");"""),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    h(3, "Beyond the basics"),
    md("- **Invalidation** is the hard part — set a sensible **TTL** and bust the key on "
       "write. *\"There are only two hard things: cache invalidation and naming things.\"*\n"
       "- **Cache tags** (Redis) group related keys so you can flush a whole set at once.\n"
       "- **`Cache::lock()`** — a distributed lock to stop a **thundering herd** (many "
       "requests rebuilding the same expensive value on a cold cache).\n"
       "- Redis also powers **queues, sessions, and rate limiting** — one dependency, "
       "several jobs.\n"
       "- Cache **IDs / small DTOs**, not giant object graphs; watch memory and eviction "
       "policy (`allkeys-lru`)."),
    code("php",
"""Cache::lock("rebuild:home-feed", 10)->get(function () {
    return Cache::put('home-feed', $this->buildExpensiveFeed(), 300);
});"""),
]))

# ===========================================================================
# 17. Queues & Horizon
# ===========================================================================
pages.append(page([
    h(2, "17 · Queues & Background Jobs"),
    why("Job queues are a stated must-have. Anything slow — emails, image resizing, "
        "calling Stripe/OpenAI, exports — must not block the HTTP response. This is core "
        "backend signal."),
    md("A **producer** dispatches a job onto a **broker** (Redis / SQS); a separate "
       "**worker** process pulls and runs it asynchronously. In production you run "
       "**Laravel Horizon** to supervise Redis queues with dashboards and metrics."),
    code("php",
"""// Producer — returns instantly, user isn't kept waiting
ProcessPayout::dispatch($order)->onQueue('payments');

// The job
class ProcessPayout implements ShouldQueue {
    public int $tries = 3;
    public array $backoff = [10, 30, 60];   // retry with backoff

    public function handle(PayoutService $svc): void {
        $svc->pay($this->orderId);          // pass an ID, not the whole model
    }
}"""),
    columns([
        slot([
            h(4, "GET RIGHT"),
            md("- **Idempotent** jobs — safe to run twice\n"
               "- **Retries** with backoff; a **failed_jobs** table / dead-letter for "
               "poison jobs\n"
               "- Pass **IDs, not whole objects** (models re-fetch fresh)\n"
               "- **Batching & chaining** for multi-step workflows"),
        ], bg=CREAM),
        slot([
            h(4, "BATCHING"),
            code("php",
"""Bus::batch([
    new ImportChunk(1),
    new ImportChunk(2),
])->then(fn() => Notify::dispatch())
  ->onQueue('imports')
  ->dispatch();"""),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    say("**Say this:** *\"I push slow or third-party work to a queue and keep jobs "
        "idempotent with retry-and-backoff, so a flaky external API becomes a retry, not "
        "a failed request. Horizon gives me visibility over throughput and failures.\"*"),
]))

# ===========================================================================
# 18. Events & Listeners
# ===========================================================================
pages.append(page([
    h(2, "18 · Events, Listeners & Observers"),
    md("Events let one thing happen and **many** things react — without coupling them "
       "together. Perfect for the *\"when an order is placed, do X, Y and Z\"* problems a "
       "marketplace is full of."),
    code("php",
"""// The order just announces what happened
OrderPlaced::dispatch($order);

// Listeners subscribe independently (queue the slow ones)
class SendReceipt implements ShouldQueue { public function handle(OrderPlaced $e) {...} }
class ReserveStock                        { public function handle(OrderPlaced $e) {...} }
class NotifySeller  implements ShouldQueue { public function handle(OrderPlaced $e) {...} }"""),
    columns([
        slot([
            h(4, "WHY DECOUPLE"),
            md("Adding a new reaction (analytics, fraud check) means adding a *listener*, "
               "not editing the checkout code. Open/closed in practice — and each "
               "listener is independently testable and queueable."),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "vs OBSERVERS vs JOBS"),
            md("- **Observer** — tied to model lifecycle (`created`, `deleted`)\n"
               "- **Event/Listener** — domain events, many reactions\n"
               "- **Job** — a single unit of deferred work\n\n"
               "They compose: an event's listener *dispatches a job*."),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    section([
        h(4, "DON'T OVERDO IT"),
        md("Events add indirection. For a simple, single side-effect, a direct call in a "
           "service is clearer. Reach for events when there are genuinely *several* "
           "independent reactions — say that trade-off and you sound senior."),
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# Part Three divider — APIs, Ops & Modernising
# ===========================================================================
pages.append(page([
    h(2, "Part Three · APIs, Ops & Modernising"),
    section([
        md("They integrate with **Stripe, Google, AWS and OpenAI**, run at marketplace "
           "scale, and the job itself is to **modernise their Laravel app**. This part is "
           "the integration, testing, security and DevOps layer — plus how to talk about "
           "improving a large legacy codebase without breaking it."),
    ], bg=RED, tc=RED_T, pad="22pt"),
    columns([
        slot([
            h(4, "INTEGRATIONS"),
            md("19. Third-party APIs\n20. Webhooks & Idempotency\n21. Microservices"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "QUALITY & SECURITY"),
            md("22. Testing Laravel\n23. Security"),
        ], bg=CREAM),
        slot([
            h(4, "RUNNING IT"),
            md("24. DevOps & Backups\n25. Modernising a Legacy App\n26. The Problem-Solver"),
        ], bg=WHITE, border={"width": "1pt", "color": LINE}),
    ], ratios=[1, 1, 1], gutter="6mm"),
]))

# ===========================================================================
# 19. Third-party APIs
# ===========================================================================
pages.append(page([
    h(2, "19 · Third-party APIs"),
    why("\"Worked with many APIs, e.g. Stripe, Google, AWS, OpenAI\" is a must-have. Show "
        "you integrate *robustly* — timeouts, retries, wrapping the vendor behind your "
        "own interface — not just calling a URL and hoping."),
    code("php",
"""use Illuminate\\Support\\Facades\\Http;

$response = Http::withToken(config('services.openai.key'))
    ->timeout(15)
    ->retry(3, 200, throw: false)          // 3 tries, 200ms apart
    ->post('https://api.openai.com/v1/chat/completions', [
        'model'    => 'gpt-...',
        'messages' => $messages,
    ]);

if ($response->failed()) {
    Log::warning('OpenAI call failed', ['status' => $response->status()]);
    return $this->fallback();               // degrade gracefully
}"""),
    columns([
        slot([
            h(4, "WRAP THE VENDOR (ADAPTER)"),
            md("Hide the SDK behind *your* interface so a vendor change or price hike "
               "stays contained in one class — and you can fake it in tests. This is the "
               "Adapter pattern, and it's exactly the mindset behind their *\"Google Maps "
               "raised fees 100x, find a workaround\"* question (§26)."),
            code("php",
"""interface Geocoder {
  public function locate(string $q): LatLng;
}
class GoogleGeocoder implements Geocoder {}
class MapboxGeocoder implements Geocoder {}"""),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "ROBUSTNESS CHECKLIST"),
            md("- **Timeouts** on every call — never hang a request\n"
               "- **Retry with backoff** on transient failures\n"
               "- **Circuit breaker / fallback** when a provider is down\n"
               "- Do it in a **queued job** so a slow API doesn't block the user\n"
               "- **Cache** responses where allowed (geocoding, rates)\n"
               "- Keys in **`.env` / config**, never in code"),
        ], bg=CREAM),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# 20. Webhooks & Idempotency
# ===========================================================================
pages.append(page([
    h(2, "20 · Webhooks & Idempotency"),
    why("Stripe (and most payment/marketplace flows) push events to you via **webhooks**. "
        "Handling them safely — verifying signatures and processing exactly once — is a "
        "very common senior interview probe."),
    code("php",
"""public function handle(Request $request)
{
    // 1. VERIFY the signature — never trust the payload blindly
    $event = Webhook::constructEvent(
        $request->getContent(),
        $request->header('Stripe-Signature'),
        config('services.stripe.webhook_secret'),
    );

    // 2. IDEMPOTENCY — Stripe may deliver the same event twice
    if (ProcessedEvent::where('event_id', $event->id)->exists()) {
        return response()->noContent();   // already handled
    }

    // 3. Do the minimum inline, queue the rest
    ProcessedEvent::create(['event_id' => $event->id]);
    HandleStripeEvent::dispatch($event->id);

    // 4. Return 2xx FAST so the provider stops retrying
    return response()->noContent();
}"""),
    columns([
        slot([
            h(4, "THE FOUR RULES"),
            md("1. **Verify the signature** (HMAC) before anything\n"
               "2. **Be idempotent** — store the event id, ignore duplicates\n"
               "3. **Return 2xx quickly** — offload work to a queue\n"
               "4. **Expect retries & out-of-order** delivery"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "IDEMPOTENCY KEYS (OUTBOUND)"),
            md("The mirror image: when *you* call Stripe to charge, send an "
               "**Idempotency-Key** so a retry after a timeout doesn't double-charge the "
               "customer. Same principle, both directions."),
        ], bg=CREAM),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# 21. Microservices
# ===========================================================================
pages.append(page([
    h(2, "21 · Microservices & SOA"),
    why("Listed under *\"would be lovely\"*. You don't need to have run a fleet — but "
        "know the trade-offs and, crucially, be honest about when a **modular monolith** "
        "is the better call. That nuance is the senior answer."),
    columns([
        slot([
            h(4, "WHAT & WHY"),
            md("Independent services, each owning its data, talking over HTTP/queues. "
               "Benefits: independent deploy & scaling, team autonomy, fault isolation. A "
               "payments service can scale separately from search."),
        ], bg=CREAM),
        slot([
            h(4, "THE REAL COSTS"),
            md("- **Distributed transactions** are hard (no cross-service ACID → sagas)\n"
               "- **Network latency** & partial failure everywhere\n"
               "- **Operational overhead** — observability, deploys, versioning\n"
               "- Data consistency becomes **eventual**"),
        ], bg=BAD, tc=BAD_T),
    ], ratios=[1, 1]),
    h(3, "The pragmatic view"),
    md("Most teams are better served by a **well-structured modular monolith** first — "
       "clear domain boundaries, a clean service layer — then extract a service *only "
       "when a real scaling or team boundary demands it*. Communication between services "
       "leans on **queues/events** (async, resilient) over synchronous chains where "
       "possible."),
    section([
        md("**Say this:** *\"I'd extract a service when there's a genuine scaling or "
           "ownership boundary — not by default. Premature microservices trade one hard "
           "problem for a dozen distributed ones. A modular monolith with clear "
           "boundaries gets you most of the benefit at a fraction of the cost.\"*")
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 22. Testing Laravel
# ===========================================================================
pages.append(page([
    h(2, "22 · Testing Laravel"),
    why("Modernising a legacy app *safely* is impossible without tests — they're the "
        "safety net that lets you refactor with confidence. Expect to be asked how you'd "
        "add coverage to code that has none."),
    columns([
        slot([
            h(4, "THE PYRAMID"),
            md("Many fast **unit** tests, fewer **feature/integration** tests (real DB), "
               "fewest **end-to-end**. Laravel's **feature tests** hit real routes through "
               "the framework — brilliant value for confidence per line."),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "THE TOOLS"),
            md("- **Pest** or **PHPUnit**\n"
               "- **Factories** — generate model data\n"
               "- **RefreshDatabase** — clean DB per test\n"
               "- **Http::fake()** — stub external APIs\n"
               "- **Queue::fake() / Event::fake()** — assert dispatch\n"
               "- **Bus::fake()**, **Mail::fake()**, `travel()` for time"),
        ], bg=CREAM),
    ], ratios=[1, 1]),
    code("php",
"""it('places an order and charges the customer', function () {
    Http::fake(['api.stripe.com/*' => Http::response(['id' => 'ch_1'], 200)]);
    Queue::fake();

    $user = User::factory()->create();
    $product = Product::factory()->create(['stock' => 5]);

    $this->actingAs($user)
        ->postJson('/api/v1/orders', ['items' => [['sku' => $product->sku, 'qty' => 1]]])
        ->assertCreated();

    expect($product->fresh()->stock)->toBe(4);
    Queue::assertPushed(SendReceipt::class);
});"""),
    say("**Say this:** *\"On a legacy app I start with feature tests around the critical "
        "money paths — they pin current behaviour so I can refactor underneath without "
        "fear. If something's hard to test, that's usually the design telling me to "
        "inject a dependency.\"*"),
]))

# ===========================================================================
# 23. Security
# ===========================================================================
pages.append(page([
    h(2, "23 · Security in Laravel"),
    md("Laravel handles a lot for you — but knowing *what* it protects against, and where "
       "the gaps are, matters on an app handling millions of users' data and payments."),
    columns([
        slot([
            h(4, "MOSTLY HANDLED — KNOW WHY"),
            md("- **SQL injection** — the query builder / Eloquent **bind** parameters "
               "(don't concatenate into `DB::raw`)\n"
               "- **XSS** — Blade `{{ }}` escapes output (`{!! !!}` does not)\n"
               "- **CSRF** — the `@csrf` token on web forms\n"
               "- **Passwords** — `Hash::make()` (bcrypt/argon), never plain/MD5"),
        ], bg=GOOD, tc=GOOD_T),
        slot([
            h(4, "YOUR RESPONSIBILITY"),
            md("- **Mass assignment** — set `$fillable`/`$guarded` deliberately\n"
               "- **Authorization** — Policies & Gates, not just authentication\n"
               "- **Rate limiting** on auth and API routes\n"
               "- **Secrets in `.env`**, never committed; `.env` in `.gitignore`\n"
               "- **`APP_DEBUG=false`** in prod — a stack trace is a data leak"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    code("php",
"""// SAFE — bound parameter, even in a raw fragment
User::whereRaw('email = ?', [$email])->first();

// Authorization, not just auth
$this->authorize('update', $order);   // runs OrderPolicy@update"""),
    section([
        h(4, "MULTI-TENANCY / DATA ISOLATION"),
        md("On a marketplace, the classic breach is one seller seeing another's data. "
           "**Scope every query by the owner** — a global scope or policy makes correct "
           "the *default*, not something each developer must remember."),
    ], bg=BAD, tc=BAD_T),
]))

# ===========================================================================
# 24. DevOps & Backups
# ===========================================================================
pages.append(page([
    h(2, "24 · DevOps & Backups"),
    why("Listed as *\"would be lovely\"* — they even name *\"taking a MySQL backup\"*. You "
        "don't need to be an SRE; show you can deploy safely and wouldn't be helpless if "
        "the database needed restoring."),
    columns([
        slot([
            h(4, "MYSQL BACKUP & RESTORE"),
            code("bash",
"""# logical dump (portable)
mysqldump -u user -p --single-transaction \\
  --routines myapp > backup.sql

# restore
mysql -u user -p myapp < backup.sql

# large DBs: physical/binary backup
# (Percona XtraBackup) + binlog for PITR"""),
            md("`--single-transaction` gives a consistent snapshot without locking InnoDB "
               "tables. **Point-in-time recovery** replays the **binlog** on top of the "
               "last full backup."),
        ], bg=CREAM),
        slot([
            h(4, "DEPLOY & RUN"),
            md("- **Docker** for parity: same image local → prod\n"
               "- **CI** runs tests before merge; **zero-downtime deploy** (Envoyer / "
               "Deployer / rolling)\n"
               "- On deploy: `migrate --force`, **clear OPcache**, `config:cache`, restart "
               "queue workers\n"
               "- **12-factor config** — differences live in `.env`, never in code\n"
               "- **Backups are only real if you've tested a restore**"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
    section([
        h(4, "OBSERVABILITY"),
        md("You can't step-debug prod. Lean on **structured logs** (with a request id), "
           "**metrics** (error rate, p95 latency), and **APM traces** (New Relic / "
           "Datadog / Sentry). In an incident: **roll back first**, diagnose from the "
           "signals second."),
    ], bg=INK, tc=INK_T),
]))

# ===========================================================================
# 25. Modernising a Legacy App
# ===========================================================================
pages.append(page([
    h(2, "25 · Modernising a Legacy Laravel App"),
    why("This *is* the job — *\"assist in modernising the Laravel app\"*. Have a clear, "
        "safe, incremental method. The instinct they're screening *against* is the "
        "big-bang rewrite."),
    section([
        h(4, "THE METHOD — STRANGLE, DON'T REWRITE"),
        md("1. **Measure & map** — where's the pain (slow pages, error rates, fat "
           "classes)? Don't guess.\n"
           "2. **Get a safety net** — feature tests around the critical paths first.\n"
           "3. **Upgrade the foundation** — get onto a supported **PHP** & **Laravel** "
           "version (Rector automates much of it); free performance and security.\n"
           "4. **Refactor incrementally** — fat controllers → services; magic strings → "
           "enums; N+1s → eager loads; add indexes where EXPLAIN says so.\n"
           "5. **Strangle** — build new features cleanly alongside the old, migrate "
           "traffic gradually, delete the old path once nothing uses it."),
    ], bg=INK, tc=INK_T),
    columns([
        slot([
            h(4, "QUICK WINS TO NAME"),
            md("- Turn on **OPcache**, add missing **indexes**\n"
               "- Kill the worst **N+1s** (Debugbar finds them)\n"
               "- Move slow work to **queues**\n"
               "- Add **caching** to the hottest read paths\n"
               "- Upgrade PHP — often the cheapest speedup there is"),
        ], bg=CREAM),
        slot([
            h(4, "WHY NOT A REWRITE"),
            md("A full rewrite means months with no new value, high risk, and a real "
               "chance of reproducing the same bugs. Incremental modernisation ships "
               "value continuously and keeps the business running — exactly what a "
               "*\"fast-growing\"* company needs."),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# 26. The Problem-Solver
# ===========================================================================
pages.append(page([
    h(2, "26 · The Problem-Solver Mindset"),
    why("They explicitly want someone who *solves problems* rather than works tickets — "
        "and gave the exact example: *\"Google Maps increased fees by 100x, find a "
        "workaround.\"* Have a structured way to answer this class of question."),
    section([
        h(4, "A FRAMEWORK FOR THE 'FIND A WORKAROUND' QUESTION"),
        md("1. **Clarify the real constraint** — is it cost, a hard rate limit, an outage? "
           "What's the actual budget / SLA?\n"
           "2. **Quantify** — how many calls, how often, how much are we really spending? "
           "Measure before acting.\n"
           "3. **Reduce demand** — **cache** results, **batch** requests, debounce, "
           "precompute. Often 90% of calls are avoidable.\n"
           "4. **Find alternatives** — is there a cheaper provider (Mapbox, OpenStreetMap "
           "/ Nominatim), a self-hosted option, or a good-enough offline dataset?\n"
           "5. **Abstract the vendor** — because you wrapped it behind an interface (§19), "
           "swapping providers is a config change, not a rewrite.\n"
           "6. **Trade-offs, out loud** — cost vs accuracy vs effort. Recommend one, and "
           "say why."),
    ], bg=INK, tc=INK_T),
    columns([
        slot([
            h(4, "WHAT THEY'RE LISTENING FOR"),
            md("Not a memorised answer — a *thinking process*: clarify → measure → reduce "
               "→ alternatives → trade-offs. Curiosity, pragmatism, and cost-awareness. "
               "Exactly the traits the ad keeps repeating."),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(4, "THE ONE-LINER"),
            quote("First I'd measure what we're actually spending and why, then attack "
                  "demand with caching and batching before I even look at alternatives — "
                  "and because the provider sits behind our own interface, switching is a "
                  "config change, not a rewrite."),
        ], bg=CREAM),
    ], ratios=[1, 1]),
]))

# ===========================================================================
# Quick Reference + Questions to ask them + Closing
# ===========================================================================
pages.append(page([
    h(2, "Quick Reference · Phrases to Use"),
    md("Drop these into answers — they signal you think about *why*, not just *how*."),
    table(
        ["Topic", "Key phrase"],
        [
            ["N+1", "Eager load relationships rather than lazy loading in a loop; preventLazyLoading in dev."],
            ["Indexes", "Covering & composite, equality-first — verify with EXPLAIN before adding."],
            ["Slow query", "First move is EXPLAIN — check for a full scan or filesort."],
            ["Big data", "Chunk or cursor; never load a million rows into memory. Keyset over deep OFFSET."],
            ["Transactions", "Wrap related writes so they all commit or all roll back — lockForUpdate for races."],
            ["Caching", "Cache-aside with a TTL; the hard part is invalidation on write."],
            ["Queues", "Push slow/third-party work to a queue; keep jobs idempotent with backoff."],
            ["Webhooks", "Verify the signature, be idempotent, return 2xx fast, queue the rest."],
            ["APIs", "Timeouts + retries, wrap the vendor behind an interface, degrade gracefully."],
            ["DI", "Program to interfaces; the container injects — loose coupling and testability."],
            ["Thin controllers", "Validate in a form request, do work in a service, return a resource."],
            ["Microservices", "Extract on a real boundary, not by default — a modular monolith first."],
            ["Modernising", "Strangler, not rewrite — safety net, upgrade the base, refactor incrementally."],
            ["Testing", "Feature-test the money paths first; hard to test usually means bad design."],
            ["Multi-tenancy", "Scope every query by owner by default, not by memory."],
        ],
    ),
    columns([
        slot([
            h(4, "ASK THEM (SHOWS INTEREST)"),
            md("- How big is the largest table, and where does it hurt today?\n"
               "- What does the modernisation roadmap look like — versions, tech debt?\n"
               "- Monolith or services now, and where do you want to be?\n"
               "- How do you handle deploys, backups and incidents?\n"
               "- What does success in the first 90 days look like?"),
        ], bg=CARD, border={"width": "1pt", "color": CARD_B}),
        slot([
            h(3, "You've got this, Chris"),
            md("You build and ship real production systems and can explain your thinking "
               "clearly — that's most of the battle. Read the *Why it matters* boxes once "
               "more, tie every answer back to *their* marketplace, and go have a "
               "conversation, not an exam."),
        ], bg=INK, tc=INK_T),
    ], ratios=[1, 1]),
]))

# ---- assemble --------------------------------------------------------------
doc = {
    "frontmatter": {
        "title": "Laravel Interview Revision Guide",
        "subtitle": "Senior Backend Laravel Developer — Marketplace",
        "author": "Chris Garlick",
        "recipient": "Interview Preparation",
        "date": "2026-07-01",
        "document_type": "brief",
        "client": "chris-garlick-light",
    },
    "pages": pages,
}

with open("php-myjobquote.json", "w") as f:
    json.dump(doc, f, indent=2, ensure_ascii=False)

print(f"Wrote php-myjobquote.json — {len(pages)} content pages")
