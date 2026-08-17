import { config } from '@fohte/eslint-config'

<<<<<<< before updating
export default config(
  {
    typescript: { typeChecked: true },
    errorHandling: {},
  },
  ...storybook.configs['flat/recommended'],
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
)
||||||| last update
export default config(
  {
    typescript: { typeChecked: true },
    errorHandling: {},
  },
  ...storybook.configs['flat/recommended'],
)
=======
export default config({
  typescript: { typeChecked: true },
  errorHandling: {},
})
>>>>>>> after updating
