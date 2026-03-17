import type { Preview } from '@storybook/react-vite'

const preview: Preview = {
  parameters: {
    backgrounds: {
      default: 'dark',
      values: [{ name: 'dark', value: '#000000' }],
    },
  },
  decorators: [
    (Story) => (
      <div
        style={{
          backgroundColor: '#000000',
          color: '#ffffff',
          fontFamily:
            "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
          minHeight: '100vh',
          padding: '16px',
        }}
      >
        <Story />
      </div>
    ),
  ],
}

export default preview
