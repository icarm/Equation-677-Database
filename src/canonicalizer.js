import { Container } from '@cloudflare/containers'

export class Canonicalizer extends Container {
  defaultPort = 8080
  // Inactivity window before the container is stopped. The timer is renewed
  // when a request arrives, not while one is being served, so this is also an
  // effective ceiling on a single canonicalization: a /canonicalize call that
  // runs longer than this is killed mid-flight and surfaces to the submitter
  // as "Container suddenly disconnected, try again".
  //
  // Canonicalization is not a function of n alone. canonicalize2_with_perm
  // builds a graph with one vertex per element plus one per cell — 742k
  // vertices and 2.2M edges at n = 861 — and Traces' runtime depends on how
  // fast refinement discriminates them, so a highly symmetric magma can cost
  // far more than a larger asymmetric one. At 10m, an order-861 direct product
  // timed out twice while orders 961–973 are already in the database.
  sleepAfter = '1h'
}
