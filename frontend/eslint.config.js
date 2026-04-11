import { config } from '@fohte/eslint-config'
import storybook from 'eslint-plugin-storybook'

export default config(
  { typescript: { typeChecked: true } },
<<<<<<< before updating
  {
    files: ['**/*.ts{,x}'],
    languageOptions: {
      parserOptions: {
        projectService: {
          allowDefaultProject: ['.storybook/*.ts', '.storybook/*.tsx'],
        },
      },
    },
  },
=======
  ...storybook.configs['flat/recommended'],
>>>>>>> after updating
  {
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['./*', '../*'],
              message:
                'Please use absolute imports instead of relative imports.',
            },
          ],
        },
      ],
    },
  },
  {
    files: ['.storybook/**', 'vitest.config.ts'],
    rules: {
      'no-restricted-imports': 'off',
    },
  },
)
