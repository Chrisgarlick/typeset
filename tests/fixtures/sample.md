---
title: Project Proposal
subtitle: Q3 Digital Strategy
recipient: Acme Corp
date: 2026-05-08
author: Chris Garlick
document_type: proposal
client: acme
---

# Executive Summary

This proposal outlines our recommended digital strategy for Q3 2026. We have identified three key opportunities that will drive growth and improve customer engagement across all channels.

## Key Objectives

1. Increase organic traffic by 40% through targeted content marketing
2. Launch the new customer portal with self-service capabilities
3. Implement real-time analytics dashboard for stakeholder reporting

## Current Performance

| Metric | Q1 2026 | Q2 2026 | Target Q3 |
|--------|---------|---------|-----------|
| Monthly Active Users | 12,400 | 15,800 | 22,000 |
| Conversion Rate | 2.1% | 2.8% | 3.5% |
| Avg Session Duration | 3m 20s | 4m 10s | 5m+ |
| NPS Score | 42 | 48 | 55+ |

## Proposed Approach

Our approach is built on three pillars:

- **Content-led growth**: Publishing high-value articles targeting long-tail keywords in the fintech space
- **Product-led acquisition**: Free-tier features that demonstrate value before requiring signup
- **Community building**: Slack community and monthly webinars to drive retention

> "The best marketing doesn't feel like marketing." — Tom Fishburne

### Technical Requirements

The implementation requires the following infrastructure changes:

```yaml
services:
  analytics:
    image: plausible/analytics:v2
    ports:
      - "8000:8000"
    environment:
      - DATABASE_URL=postgresql://...
      - SECRET_KEY_BASE=...

  portal:
    build: ./portal
    ports:
      - "3000:3000"
```

### Timeline

1. **Week 1-2**: Infrastructure setup and migration planning
2. **Week 3-4**: Portal MVP development and internal testing
3. **Week 5-6**: Content production sprint (12 articles)
4. **Week 7-8**: Soft launch and monitoring
5. **Week 9-10**: Full rollout with marketing push
6. **Week 11-12**: Optimization and Q4 planning

---

## Budget Breakdown

| Category | Monthly Cost | Q3 Total |
|----------|-------------|----------|
| Infrastructure | £2,400 | £7,200 |
| Content Production | £4,000 | £12,000 |
| Analytics Tools | £800 | £2,400 |
| Community Platform | £200 | £600 |
| **Total** | **£7,400** | **£22,200** |

## Next Steps

We recommend scheduling a kickoff meeting for the week of May 12th to align on priorities and assign workstreams. Please review this proposal and share any feedback before then.

---

*Prepared by Chris Garlick · May 2026*
