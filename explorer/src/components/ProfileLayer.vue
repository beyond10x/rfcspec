<script setup lang="ts">
import { computed } from 'vue';
import type { RfcspecProfileExplorerProfile as Profile } from '../generated-profile/types.ts';
import {
  GAPS,
  byLevel,
  decisionCounts,
  decisionsByItem,
  downloadHref,
  overlayId,
  profilePage,
  reviewedByLevel,
  sourceHref,
  statusCounts,
  total,
} from '../lib/profile.ts';

const props = defineProps<{ profile: Profile }>();

const statuses = computed(() => statusCounts(props.profile));
const decisions = computed(() => decisionCounts(props.profile));
/** Each requirement with the decision on its item, joined on the item id. */
const rows = computed(() => {
  const byItem = decisionsByItem(props.profile);
  return props.profile.requirements.map((requirement) => ({
    requirement,
    decision: byItem.get(requirement.status.requirement_id),
  }));
});
const rejected = computed(() => props.profile.decisions.filter((decision) => decision.row.kind === 'reject'));

/** The unreviewed line, worded as the profile page words it. */
const unreviewed = computed(() => {
  const counts = props.profile.unreviewed;
  const rfc = props.profile.rfc;
  if (counts === undefined) {
    return `Unreviewed: not counted, because the profile holds no locked source of RFC ${rfc}.`;
  }
  if (counts.length === 0) {
    return `Unreviewed: none. Every requirement the mechanical catalogue of RFC ${rfc} holds has a status.`;
  }
  return `Unreviewed: ${total(counts)} requirements of the mechanical catalogue of RFC ${rfc} have no status, by level ${byLevel(counts)}.`;
});

const STATUS: Record<string, string> = {
  satisfied: 'satisfied',
  unsupported: 'unsupported',
  'not-applicable': 'not applicable',
};
</script>

<template>
  <section data-test="reviewed-layer" :data-profile="profile.profile">
    <h2>Reviewed layer: profile {{ profile.profile }}</h2>
    <p class="note">
      The requirements, review decisions, machines and layouts of
      <a :href="sourceHref(`${profile.path}/overlay.yaml`)"><code>{{ profile.path }}/overlay.yaml</code></a>,
      as the <a :href="profilePage(profile.profile)">{{ profile.profile }} profile page</a> shows them.
    </p>
    <p data-test="counts">
      Reviewed: {{ profile.requirements.length }} requirements, by level {{ reviewedByLevel(profile) }}; by status
      {{ statuses.satisfied }} satisfied, {{ statuses.unsupported }} unsupported, {{ statuses.notApplicable }} not
      applicable. {{ unreviewed }}
    </p>
    <p data-test="decisions">
      <template v-if="profile.decisions.length === 0">No review decision on a recorded proposal.</template>
      <template v-else>
        Review decisions on recorded proposals: {{ decisions.decided }} accepted or edited ({{ decisions.accept }}
        accept, {{ decisions.edit }} edit), {{ decisions.rejected }} rejected.
      </template>
    </p>

    <h3>Requirements</h3>
    <table data-test="requirements">
      <thead>
        <tr>
          <th>Id</th><th>Level</th><th>Transports</th><th>Quote</th><th>Source</th><th>Status</th>
          <th>Element or reason</th><th>ESS gap</th><th>Decision</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="{ requirement, decision } in rows"
          :key="requirement.status.requirement_id"
          :data-item="requirement.status.requirement_id"
        >
          <td>{{ overlayId(requirement.status.requirement_id) }}</td>
          <td>{{ requirement.level }}</td>
          <td>{{ requirement.transports.length === 0 ? 'all' : requirement.transports.join(', ') }}</td>
          <td>{{ requirement.quote }}</td>
          <td>lines {{ requirement.start_line }}-{{ requirement.end_line }}</td>
          <td>{{ STATUS[requirement.status.kind] }}</td>
          <td>{{ requirement.status.element ?? requirement.status.reason ?? '' }}</td>
          <td><a v-if="requirement.gap !== undefined" :href="GAPS">G{{ requirement.gap }}</a></td>
          <td>
            <template v-if="decision !== undefined">
              <span data-test="decision">{{ decision.row.kind }}</span>
              <template v-if="decision.proposal_path !== undefined">
                of proposal
                <a :href="sourceHref(decision.proposal_path)"
                  ><code>{{ decision.row.proposal_id }}</code></a
                >
                by <code>{{ decision.model_id }}</code>
              </template>
            </template>
          </td>
        </tr>
      </tbody>
    </table>

    <template v-if="rejected.length > 0">
      <h3>Rejected proposals</h3>
      <p class="note">Each rejected item stays unreviewed and is not proposed again.</p>
      <table data-test="rejected">
        <thead>
          <tr><th>Item</th><th>Reason</th><th>Proposal</th><th>Model</th></tr>
        </thead>
        <tbody>
          <tr v-for="decision in rejected" :key="decision.row.key.item">
            <td>{{ decision.row.key.item }}</td>
            <td>{{ decision.row.reason }}</td>
            <td>
              <a v-if="decision.proposal_path !== undefined" :href="sourceHref(decision.proposal_path)"
                ><code>{{ decision.row.proposal_id }}</code></a
              >
            </td>
            <td><code v-if="decision.model_id !== undefined">{{ decision.model_id }}</code></td>
          </tr>
        </tbody>
      </table>
    </template>

    <h3>Machines</h3>
    <p v-if="profile.machines.length === 0">No machine.</p>
    <table v-else data-test="machines">
      <thead>
        <tr><th>Machine</th><th>Transport</th><th>Model</th><th>SHA-256</th><th>Input witness</th></tr>
      </thead>
      <tbody>
        <tr v-for="machine in profile.machines" :key="machine.name">
          <td>{{ machine.name }}</td>
          <td>{{ machine.transport }}</td>
          <td><a :href="downloadHref(machine.download)"><code>{{ machine.name }}.protospec.yaml</code></a></td>
          <td><code>{{ machine.protospec_digest }}</code></td>
          <td><a :href="sourceHref(machine.witness)"><code>{{ machine.witness }}</code></a></td>
        </tr>
      </tbody>
    </table>

    <h3>Layouts</h3>
    <p v-if="profile.layouts.length === 0">No layout.</p>
    <template v-for="layout in profile.layouts" :key="layout.row.layout_id">
      <h4>{{ layout.row.name }}</h4>
      <table :data-test="`layout-${layout.row.name}`">
        <thead>
          <tr><th>Field</th><th>Bits</th><th>Present when</th><th>Constraints</th></tr>
        </thead>
        <tbody>
          <tr v-for="field in layout.fields" :key="field.name">
            <td>{{ field.name }}</td>
            <td>{{ field.bits ?? '' }}</td>
            <td>{{ field.presence === undefined ? 'always' : `${field.presence.field} = ${field.presence.equals}` }}</td>
            <td>{{ field.constraints.map((c) => `${c.kind} ${c.values.join(', ')}`).join('; ') }}</td>
          </tr>
        </tbody>
      </table>
    </template>
  </section>
</template>
