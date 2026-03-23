import type { Meta, StoryObj } from '@storybook/react-vite'

import { UpdateNotificationBar } from '@/components/UpdateNotificationBar'

const meta = {
  title: 'Components/UpdateNotificationBar',
  component: UpdateNotificationBar,
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof UpdateNotificationBar>

export default meta
type Story = StoryObj<typeof meta>

const noop = () => {}

export const Available: Story = {
  args: {
    state: { status: 'available', version: '1.2.0' },
    onUpdate: noop,
    onDismiss: noop,
  },
}

export const DownloadingStart: Story = {
  args: {
    state: { status: 'downloading', progress: 0 },
    onUpdate: noop,
    onDismiss: noop,
  },
}

export const DownloadingInProgress: Story = {
  args: {
    state: { status: 'downloading', progress: 45 },
    onUpdate: noop,
    onDismiss: noop,
  },
}

export const DownloadingAlmostDone: Story = {
  args: {
    state: { status: 'downloading', progress: 99 },
    onUpdate: noop,
    onDismiss: noop,
  },
}

export const Error: Story = {
  args: {
    state: { status: 'error', message: 'Network request failed' },
    onUpdate: noop,
    onDismiss: noop,
  },
}
