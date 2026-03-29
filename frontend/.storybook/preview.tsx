import '../src/font-size.css'

import type { Preview } from '@storybook/react-vite'

const preview: Preview = {
  parameters: {
    backgrounds: {
      disable: true,
    },
  },
  decorators: [
    (Story) => (
      <div
        style={{
          backgroundImage: [
            'linear-gradient(45deg, #222 25%, transparent 25%)',
            'linear-gradient(-45deg, #222 25%, transparent 25%)',
            'linear-gradient(45deg, transparent 75%, #222 75%)',
            'linear-gradient(-45deg, transparent 75%, #222 75%)',
          ].join(', '),
          backgroundSize: '20px 20px',
          backgroundPosition: '0 0, 0 10px, 10px -10px, -10px 0',
          backgroundColor: '#333',
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
