import React from 'react'

export interface SiriOrbProps {
  /** Size in pixels or CSS units, e.g. "20px", "24px", "32px", "96px". Default "24px". */
  size?: string
  /** Class name for outer wrapper */
  className?: string
  /** Custom colors for the conic gradients */
  colors?: {
    bg?: string
    c1?: string
    c2?: string
    c3?: string
  }
  /** Animation speed in seconds. Default 10s. */
  animationDuration?: number
}

/**
 * 21st.dev Siri Orb Component
 * Siri-style animated audio visualizer orb with conic gradients,
 * smooth rotation, and customizable colors.
 * Source: https://21st.dev/community/components (id: 12161)
 */
export const SiriOrb: React.FC<SiriOrbProps> = ({
  size = '24px',
  className = '',
  colors,
  animationDuration = 8,
}) => {
  const defaultColors = {
    bg: 'oklch(95% 0.02 264.695)',
    c1: 'oklch(75% 0.18 350)',
    c2: 'oklch(80% 0.16 200)',
    c3: 'oklch(78% 0.18 280)',
  }

  const finalColors = { ...defaultColors, ...colors }
  const sizeValue = Number.parseInt(size.replace('px', ''), 10) || 24

  const blurAmount = Math.max(sizeValue * 0.02, 1.2)
  const contrastAmount = Math.max(sizeValue * 0.01, 1.3)
  const dotSize = Math.max(sizeValue * 0.008, 0.1)
  const shadowSpread = Math.max(sizeValue * 0.01, 0.8)

  return (
    <div
      className={`siri-orb shrink-0 ${className}`}
      style={
        {
          width: size,
          height: size,
          '--bg': finalColors.bg,
          '--c1': finalColors.c1,
          '--c2': finalColors.c2,
          '--c3': finalColors.c3,
          '--animation-duration': `${animationDuration}s`,
          '--blur-amount': `${blurAmount}px`,
          '--contrast-amount': contrastAmount,
          '--dot-size': `${dotSize}px`,
          '--shadow-spread': `${shadowSpread}px`,
        } as React.CSSProperties
      }
    >
      <style>{`
        @property --angle {
          syntax: "<angle>";
          inherits: false;
          initial-value: 0deg;
        }

        .siri-orb {
          display: grid;
          grid-template-areas: "stack";
          overflow: hidden;
          border-radius: 50%;
          position: relative;
        }

        .siri-orb::before,
        .siri-orb::after {
          content: "";
          display: block;
          grid-area: stack;
          width: 100%;
          height: 100%;
          border-radius: 50%;
        }

        .siri-orb::before {
          background:
            conic-gradient(
              from calc(var(--angle) * 2) at 25% 70%,
              var(--c3),
              transparent 20% 80%,
              var(--c3)
            ),
            conic-gradient(
              from calc(var(--angle) * 2) at 45% 75%,
              var(--c2),
              transparent 30% 60%,
              var(--c2)
            ),
            conic-gradient(
              from calc(var(--angle) * -3) at 80% 20%,
              var(--c1),
              transparent 40% 60%,
              var(--c1)
            ),
            conic-gradient(
              from calc(var(--angle) * 2) at 15% 5%,
              var(--c2),
              transparent 10% 90%,
              var(--c2)
            ),
            conic-gradient(
              from calc(var(--angle) * 1) at 20% 80%,
              var(--c1),
              transparent 10% 90%,
              var(--c1)
            ),
            conic-gradient(
              from calc(var(--angle) * -2) at 85% 10%,
              var(--c3),
              transparent 20% 80%,
              var(--c3)
            );
          box-shadow: inset var(--bg) 0 0 var(--shadow-spread)
            calc(var(--shadow-spread) * 0.2);
          filter: blur(var(--blur-amount)) contrast(var(--contrast-amount));
          animation: siri-orb-rotate var(--animation-duration) linear infinite;
        }

        .siri-orb::after {
          background-image: radial-gradient(
            circle at center,
            var(--bg) var(--dot-size),
            transparent var(--dot-size)
          );
          background-size: calc(var(--dot-size) * 2) calc(var(--dot-size) * 2);
          backdrop-filter: blur(calc(var(--blur-amount) * 2))
            contrast(calc(var(--contrast-amount) * 2));
          mix-blend-mode: overlay;
        }

        @keyframes siri-orb-rotate {
          to {
            --angle: 360deg;
          }
        }

        @media (prefers-reduced-motion: reduce) {
          .siri-orb::before {
            animation: none;
          }
        }
      `}</style>
    </div>
  )
}

export default SiriOrb
