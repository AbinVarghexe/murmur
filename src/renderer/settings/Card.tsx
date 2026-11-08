import type { ComponentType, ReactNode } from 'react'
import { cx } from '@/utils/cx'

type IconComponent = ComponentType<{ className?: string; 'aria-hidden'?: boolean | 'true' | 'false' }>

interface CardProps {
  readonly title: string
  readonly description?: string
  readonly icon: IconComponent
  readonly className?: string
  readonly children: ReactNode
}

/** Titled panel used by every settings section. */
export function Card({ title, description, icon: Icon, className, children }: CardProps) {
  return (
    <section
      aria-label={title}
      className={cx(
        'flex flex-col gap-4 rounded-3xl border border-border-button-default bg-background-primary-default p-5',
        className,
      )}
    >
      <header className="flex items-start gap-3">
        <span className="grid size-9 shrink-0 place-items-center rounded-xl bg-background-secondary-default text-foreground-icon-secondary">
          <Icon className="size-5" aria-hidden />
        </span>
        <div className="min-w-0">
          <h2 className="text-headline-medium text-text-primary">{title}</h2>
          {description ? <p className="text-body-regular text-text-secondary">{description}</p> : null}
        </div>
      </header>
      {children}
    </section>
  )
}
