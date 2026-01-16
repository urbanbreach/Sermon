# Milestone 03 - WASAPI Exclusive Bit-Perfect: UI Review

## Screenshots Checklist

- [ ] `settings-audio.png` - Settings view showing Audio section with Output Mode, Policy, and Fade controls
- [ ] `diagnostics-playing-44k.png` - Diagnostics view while playing a 44.1 kHz track (showing bit-perfect status)
- [ ] `diagnostics-playing-96k.png` - Diagnostics view while playing a 96 kHz track (showing format switch)

## What Changed

- Audio settings: Output Mode (Exclusive/Shared), Policy (Strict/Compatibility), Fade toggle
- Diagnostics view showing audio pipeline status
- Bit-perfect indicator in BottomBar
- Unity gain label when Exclusive+Strict mode disables volume control
- Format pipeline visualization (Source → Output)

## Verification Steps

1. Open Settings → Audio section
   - Verify Output Mode dropdown (Exclusive/Shared)
   - Verify Policy dropdown (Strict/Compatibility)
   - Verify Fade toggle checkbox
   - Capture `settings-audio.png`

2. Navigate to Diagnostics view
   - Play a 44.1 kHz track
   - Verify Bit-Perfect status shows YES/NO with reason
   - Verify format pipeline shows correct sample rates
   - Capture `diagnostics-playing-44k.png`

3. Play a 96 kHz track
   - Verify DAC indicator changes (if available)
   - Verify Diagnostics shows updated format
   - Capture `diagnostics-playing-96k.png`

## Known Issues

_None identified._

## Hardware Verification Notes

- DAC sample rate indicator behavior: _TBD_
- Exclusive mode availability: _TBD_

## Next UI Focus

- Milestone 04: Library Browse & Search
