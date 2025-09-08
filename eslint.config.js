// @ts-check
import antfu from '@antfu/eslint-config'

export default antfu({
  ignores: [
    'node_modules',
    'dist',
    'public',
    '*.d.ts',
    'src-tauri',
  ],
  rules: {
    'node/prefer-global/process': 'off',
    'ts/no-empty-object-type': 'off',
    'no-console': 'off',
  },
})
