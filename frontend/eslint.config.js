import { config } from '@fohte/eslint-config'
import storybook from 'eslint-plugin-storybook'

export default config(
  {
    typescript: { typeChecked: true },
    errorHandling: {},
  },
  ...storybook.configs['flat/recommended'],
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
||||||| last update
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
  // .storybook/ and vitest.config.ts are outside src/ where @ alias is unavailable
  {
    files: ['.storybook/**/*.ts', 'vitest.config.ts'],
    rules: {
      'no-restricted-imports': 'off',
    },
  },
=======
>>>>>>> after updating
)
