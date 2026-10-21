# Web Platform, Effect-TS Backend & Database (`apps/web`, `packages/*`)

## 1. Next.js 16 Application (`apps/web`)

- **Framework**: Next.js 16.3+ (App Router, Turbopack), React 19.2, Tailwind CSS.
- **Important Next.js 16 Note**: `apps/web/AGENTS.md` contains an auto-generated block (`<!-- BEGIN:nextjs-agent-rules -->`) managed by `next dev`. Do not strip that block from diffs, and consult `apps/web/node_modules/next/dist/docs/` for Next.js 16 API changes.
- **Key Route Groups (`apps/web/app/`)**:
  - `(site)/`: Marketing pages, pricing, features, download pages.
  - `(docs)/`: Product & self-hosting documentation.
  - `(org)/dashboard/`: Team workspaces, Spaces, Folders, Videos library, Analytics, Custom Domains, S3 / Storage integrations, WorkOS SSO, Loom importer, Developer Apps, Billing.
  - `s/[videoId]/`: Shareable video player with HLS/MP4 streaming, live/canonical interactive transcript, AI summary, chapters, timestamped comments, emoji reactions, web trim/cut (`videoEdits`), and custom Call-To-Action (`VideoCallToAction`) button.
  - `c/[collectionId]/`: Publicly shared folder collections (`PublicCollection`).
  - `embed/`: Embeddable player routes used by `@cap/sdk-embed`.
  - `mcp/`: Model Context Protocol OAuth 2.0 and AI agent tool endpoints.
  - `messenger/`: Integrated support messenger.
  - `api/`: Effect-TS `HttpApi` endpoints, upload presigning, webhooks (Stripe, WorkOS), AI generation workers.

---

## 2. Effect-TS Architecture (`packages/web-backend`, `packages/web-domain`, `apps/web`)

Cap's web backend is architected with **Effect-TS** (`effect`, `@effect/platform`, `@effect/rpc`, `@effect/sql-mysql2`):

1. **Domain Layer (`packages/web-domain`)**:
   - Defines branded IDs (`Video.VideoId`, `User.UserId`, `Organisation.OrganisationId`, `Space.SpaceId`, `Folder.FolderId`, `Comment.CommentId`, `S3Bucket.S3BucketId`, `Storage.StorageIntegrationId`, `Agent`), domain models, and typed Effect errors.
2. **Contract Layer (`packages/web-api-contract-effect` & `packages/web-api-contract`)**:
   - Declares `HttpApi`, endpoint groups, request/response schemas, and RPC contracts.
3. **Service Layer (`packages/web-backend/src/`)**:
   - Domain services implemented with `Effect.Service` / `Context.Tag` and `Layer`:
     - `Videos/`, `Organisations/`, `Spaces/`, `Folders/`, `Comments/`, `Users/`, `S3Buckets/`, `Storage/`, `ImageUploads/`, `Tinybird/`, `Extension/`, `AgentAuth.ts`, `AgentManagement.ts`, `Auth.ts`, `Aws.ts`, `Database.ts`, `Rpcs.ts`.
   - Database queries run through Drizzle wrapped in Effect services with OpenTelemetry tracing (`@kubiks/otel-drizzle`, `@effect/opentelemetry`).
4. **Next.js Route Wiring (`apps/web/app/api/*`)**:
   - **Pattern**: Build endpoints using `@effect/platform`'s `HttpApiBuilder.group`, acquire backend services inside `Effect.gen`, map domain errors to `HttpApiError` variants, and export the handler via `apiToHandler(ApiLive)` from `@/lib/server`.
   - **Server Components / Actions**: Execute effects via `EffectRuntime.runPromise` from `@/lib/server` (typically piped with `provideOptionalAuth` or `provideAuth`).
   - **Client Components**: Use `useEffectQuery` and `useEffectMutation` from `@/lib/EffectRuntime` — never call `EffectRuntime.run*` directly inside React components.

---

## 3. Database & Drizzle ORM (`packages/database`)

- **Database Engine**: MySQL 8 / PlanetScale (`drizzle-orm/mysql-core`, `mysql2`, `@planetscale/database`).
- **Complete 47-Table Schema Map (`packages/database/schema.ts`)**:
  - **Identity & Auth**: `users`, `accounts`, `sessions`, `verificationTokens`, `authApiKeys`, `signedBaas`.
  - **Organizations, SSO & Integrations**: `organizations`, `organizationSso`, `organizationMembers` (`hasProSeat`, `role`), `organizationInvites`, `integrationInstallations`.
  - **Workspaces & Hierarchy**: `spaces`, `spaceMembers`, `spaceVideos`, `folders` (supports public collections `/c/[id]`).
  - **Videos, Editing, Sharing & Engagement**: `videos` (with `VideoMetadata`, `VideoCallToAction`, `effectiveCreatedAt`, `password`), `videoEdits` (`VideoEditSpec` keepRanges), `videoUploads`, `videoProcessingJobs`, `mediaProcessingBudgets`, `sharedVideos`, `videoViewerGrants`, `comments`, `notifications`, `importedVideos`.
  - **Storage & Cloud Providers**: `s3Buckets`, `storageIntegrations` (S3, Google Drive, etc.), `storageObjects`.
  - **AI Agents & MCP OAuth**: `agentApiKeys`, `agentApiIdempotency`, `agentApiAuthorizationCodes`, `agentApiOperations`, `mcpOAuthClients`, `mcpOAuthRegistrationQuotas`, `mcpOAuthCodes`, `mcpOAuthTokens`.
  - **Developer Platform & Credits (`@cap/sdk-*`)**: `developerApps`, `developerAppDomains`, `developerApiKeys`, `developerVideos`, `developerCreditAccounts`, `developerCreditTransactions`, `developerDailyStorageSnapshots`.
  - **Support & Marketing Sync**: `messengerConversations`, `messengerMessages`, `messengerSupportEmails`, `loopsSyncJobs`.
- **Migration Workflow**:
  - **NEVER hand-edit** files inside `packages/database/migrations/`.
  - After editing `packages/database/schema.ts`:
    1. Run `bun run db:generate` to generate the Drizzle SQL migration, snapshot, and journal update.
    2. Run `bun run db:push` (in local dev) before relying on the new schema.
    3. Commit the generated files in `packages/database/migrations/` alongside `schema.ts`.
- **Emails & Marketing Sync**:
  - Transactional emails live in `packages/database/emails/` (React Email + Resend).
  - Runtime Loops profile sync lives in `packages/database/loops/` (marketing copy/journeys live in the separate `CapSoftware/cap-marketing` repo).

---

## 4. Media Server & Custom Codec (`apps/media-server` & `capcodec/`)

- **`apps/media-server`**: Lightweight Bun + Hono service handling server-side video/audio processing (`@mediabunny/server`, `node-av`, FFmpeg).
- **`capcodec/`**: Contains **Tov**, Cap's custom H.264 screen-content encoder. Activated in `apps/media-server` when `CAP_MEDIA_VIDEO_ENCODER=capcodec`. Full branch history is stored in `capcodec/capcodec.bundle`.

---

## 5. Analytics (`scripts/analytics/`)

- Powered by **Tinybird** (viewer telemetry datasources and pipes in `scripts/analytics/tinybird`) and **PostHog** / **OpenPanel**.
- Commands: `bun run analytics:check` (validate workspace), `bun run analytics:setup` (deploy Tinybird resources — destructive to unmanaged resources in the target workspace).
