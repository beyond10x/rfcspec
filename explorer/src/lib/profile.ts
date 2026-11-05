import type {
  RfcspecCatalogueLevelCount as LevelCount,
  RfcspecProfileExplorerDecision as Decision,
  RfcspecProfileExplorerProfile as Profile,
} from '../generated-profile/types.ts';

/** A file in the repository on GitHub, as the profile pages link it. */
const SOURCE = 'https://github.com/beyond10x/rfcspec/blob/main/';

/** The gaps table of the design page, as the profile pages link it. */
export const GAPS = '../docs/design#7-lowering-to-ess-and-the-gaps';

/** Why the reviewed layer of one profile mark is not shown. */
export type ProfileNote = { profile: string; text: string };

/** Where a profile's file is served, relative to the explorer page. */
export function profileUrl(name: string, page: string): URL {
  return new URL(`../data/profiles/${name}.json`, page);
}

/** A repository file on `main`, by its path from the repository root. */
export function sourceHref(path: string): string {
  return `${SOURCE}${path}`;
}

/** A download of the site, by its path from the site's base URL. */
export function downloadHref(path: string): string {
  return `../${path}`;
}

/** The profile's page in the protocol catalogue. */
export function profilePage(name: string): string {
  return `../docs/catalog/${name}`;
}

/** The overlay id of a requirement, its item id without the `rfc<N>/` prefix. */
export function overlayId(item: string): string {
  return item.replace(/^rfc[0-9]+\//, '');
}

/** Each decision by the item it judged, the join from a requirement to its decision. */
export function decisionsByItem(profile: Profile): Map<string, Decision> {
  return new Map(profile.decisions.map((decision) => [decision.row.key.item, decision]));
}

/** Requirement counts by status, as the profile page counts them. */
export function statusCounts(profile: Profile): { satisfied: number; unsupported: number; notApplicable: number } {
  const counts = { satisfied: 0, unsupported: 0, notApplicable: 0 };
  for (const requirement of profile.requirements) {
    const kind = requirement.status.kind;
    if (kind === 'satisfied') {
      counts.satisfied += 1;
    } else if (kind === 'unsupported') {
      counts.unsupported += 1;
    } else {
      counts.notApplicable += 1;
    }
  }
  return counts;
}

/** Decisions by kind: accepted or edited, accept, edit, rejected. */
export function decisionCounts(profile: Profile): { decided: number; accept: number; edit: number; rejected: number } {
  const counts = { decided: 0, accept: 0, edit: 0, rejected: 0 };
  for (const decision of profile.decisions) {
    if (decision.row.kind === 'accept') {
      counts.accept += 1;
      counts.decided += 1;
    } else if (decision.row.kind === 'edit') {
      counts.edit += 1;
      counts.decided += 1;
    } else if (decision.row.kind === 'reject') {
      counts.rejected += 1;
    }
  }
  return counts;
}

/** Counts by level as the profile page writes them: `21 MUST, 2 MUST NOT`. */
export function byLevel(counts: LevelCount[]): string {
  return counts.map((c) => `${c.count} ${c.level}`).join(', ');
}

/** The sum of counts by level. */
export function total(counts: LevelCount[]): number {
  return counts.reduce((sum, c) => sum + c.count, 0);
}

/** The BCP 14 levels in the order the profile page counts them. */
const LEVELS = ['MUST', 'MUST NOT', 'REQUIRED', 'SHALL', 'SHALL NOT', 'SHOULD', 'SHOULD NOT', 'RECOMMENDED', 'NOT RECOMMENDED', 'MAY', 'OPTIONAL'];

/** The reviewed requirements counted by level as the profile page writes them; `none` without one. */
export function reviewedByLevel(profile: Profile): string {
  const counts = new Map<string, number>();
  for (const requirement of profile.requirements) {
    counts.set(requirement.level, (counts.get(requirement.level) ?? 0) + 1);
  }
  if (counts.size === 0) {
    return 'none';
  }
  const rank = (level: string): number => {
    const found = LEVELS.indexOf(level);
    return found === -1 ? LEVELS.length : found;
  };
  return [...counts.entries()]
    .sort(([a], [b]) => rank(a) - rank(b) || (a < b ? -1 : a > b ? 1 : 0))
    .map(([level, n]) => `${n} ${level}`)
    .join(', ');
}
