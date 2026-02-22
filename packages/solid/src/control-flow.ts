type AccessorLike<T> = T | (() => T);

function resolve<T>(value: AccessorLike<T>): T {
  if (typeof value === "function") {
    return (value as () => T)();
  }
  return value;
}

export interface ShowProps<T = unknown> {
  when: AccessorLike<T>;
  fallback?: AccessorLike<unknown>;
  children?: unknown;
}

export function Show<T = unknown>(props: ShowProps<T>): () => unknown {
  return () => {
    const whenValue = resolve(props.when);
    if (whenValue) {
      if (typeof props.children === "function") {
        return (props.children as (value: T) => unknown)(whenValue);
      }
      return props.children ?? null;
    }

    if (props.fallback === undefined) {
      return null;
    }

    return resolve(props.fallback);
  };
}
