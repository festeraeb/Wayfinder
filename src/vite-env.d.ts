/// <reference types="vite/client" />

// CSS modules
declare module '*.css' {
    const content: Record<string, string>;
    export default content;
}

// CSS imports (side-effect)
declare module '*.css' {
    const css: string;
    export default css;
}
