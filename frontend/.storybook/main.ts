import type { StorybookConfig } from '@storybook/react-vite'

const config: StorybookConfig = {
  stories: ['../src/**/*.stories.@(ts|tsx)'],
<<<<<<< before updating
  addons: ['@storybook/addon-docs'],
  framework: '@storybook/react-vite',
=======
  framework: '@storybook/react-vite',
  addons: ['@storybook/addon-docs', '@storybook/addon-themes'],
>>>>>>> after updating
}
export default config
