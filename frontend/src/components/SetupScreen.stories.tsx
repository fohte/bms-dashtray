import type { Meta, StoryObj } from '@storybook/react-vite'

import { SetupScreen } from '@/components/SetupScreen'

const meta = {
  title: 'Components/SetupScreen',
  component: SetupScreen,
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof SetupScreen>

export default meta
type Story = StoryObj<typeof meta>

export const Initial: Story = {
  args: {
    selectedPath: null,
    dbFileStatuses: [],
    isValidating: false,
    error: null,
    onSelectFolder: () => {},
    onStart: () => {},
  },
}

export const PathSelected: Story = {
  args: {
    selectedPath: '/Users/player/beatoraja',
    dbFileStatuses: [],
    isValidating: true,
    error: null,
    onSelectFolder: () => {},
    onStart: () => {},
  },
}

export const ValidationSuccess: Story = {
  args: {
    selectedPath: '/Users/player/beatoraja',
    dbFileStatuses: [
      { name: 'songdata.db', found: true },
      { name: 'scoredatalog.db', found: true },
      { name: 'score.db', found: true },
      { name: 'scorelog.db', found: true },
    ],
    isValidating: false,
    error: null,
    onSelectFolder: () => {},
    onStart: () => {},
  },
}

export const ValidationError: Story = {
  args: {
    selectedPath: '/Users/player/wrong-folder',
    dbFileStatuses: [
      { name: 'songdata.db', found: true },
      { name: 'scoredatalog.db', found: false },
      { name: 'score.db', found: false },
      { name: 'scorelog.db', found: false },
    ],
    isValidating: false,
    error: 'Missing database files: scoredatalog.db, score.db, scorelog.db',
    onSelectFolder: () => {},
    onStart: () => {},
  },
}

export const AllMissing: Story = {
  args: {
    selectedPath: '/Users/player/empty-folder',
    dbFileStatuses: [
      { name: 'songdata.db', found: false },
      { name: 'scoredatalog.db', found: false },
      { name: 'score.db', found: false },
      { name: 'scorelog.db', found: false },
    ],
    isValidating: false,
    error:
      'Missing database files: songdata.db, scoredatalog.db, score.db, scorelog.db',
    onSelectFolder: () => {},
    onStart: () => {},
  },
}
