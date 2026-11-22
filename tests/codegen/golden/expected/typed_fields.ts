/**
 * A container with typed fields.
 */
export interface Container {
  id: number;
  name: string;
  running: boolean;
  tags: string[];
  exitCode: number | undefined;
  replicas: number;
  image: unknown;
}
