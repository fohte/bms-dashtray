import type { Meta, StoryObj } from '@storybook/react-vite'

import { DistributionChart } from '@/components/DistributionChart'
import { makeRecord } from '@/test-helpers'

const meta = {
  title: 'Components/DistributionChart',
  component: DistributionChart,
} satisfies Meta<typeof DistributionChart>

export default meta
type Story = StoryObj<typeof meta>

export const FewLevels: Story = {
  args: {
    records: [
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['★24'] }),
      makeRecord({ tableLevels: ['★24'] }),
      makeRecord({ tableLevels: ['sl5'] }),
    ],
  },
}

export const ManyLevels: Story = {
  args: {
    records: [
      makeRecord({ tableLevels: ['st1'] }),
      makeRecord({ tableLevels: ['st2'] }),
      makeRecord({ tableLevels: ['st2'] }),
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['sl3'] }),
      makeRecord({ tableLevels: ['sl3'] }),
      makeRecord({ tableLevels: ['★20'] }),
      makeRecord({ tableLevels: ['★20'] }),
      makeRecord({ tableLevels: ['★20'] }),
      makeRecord({ tableLevels: ['★20'] }),
      makeRecord({ tableLevels: ['★22'] }),
      makeRecord({ tableLevels: ['★22'] }),
      makeRecord({ tableLevels: ['★24'] }),
      makeRecord({ tableLevels: ['★24'] }),
      makeRecord({ tableLevels: ['★24'] }),
      makeRecord({ tableLevels: ['★25'] }),
    ],
  },
}

export const MultiTableRecords: Story = {
  args: {
    records: [
      makeRecord({ tableLevels: ['st3', 'sl5'] }),
      makeRecord({ tableLevels: ['st3', '★24'] }),
      makeRecord({ tableLevels: ['★24'] }),
    ],
  },
}

export const NoTableLevels: Story = {
  args: {
    records: [makeRecord(), makeRecord(), makeRecord()],
  },
}

export const Empty: Story = {
  args: {
    records: [],
  },
}
