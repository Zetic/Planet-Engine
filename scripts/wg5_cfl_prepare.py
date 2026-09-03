from pathlib import Path

path = Path('scripts/wg5_cfl_hardening.py')
text = path.read_text()
marker = '# Document the stability/conservation contract.\n'
assert marker in text
text = text[:text.index(marker)] + r'''# Document the stability/conservation contract.
doc = Path('docs/worldgen-rewrite/WG5_CLIMATE.md')
doc_text = doc.read_text()
needle = 'WG-5 intentionally includes a reduced B+ surface-ocean circulation model. Wind stress produces candidate currents, latitude- and rotation-rate-dependent Coriolis response deflects them, WG-4 ocean connectivity removes land-crossing flow, and bathymetry reduces shallow-water mobility. The candidate field is converted to antisymmetric ocean-interface transports and passed through a deterministic graph pressure projection so the retained transport has a small divergence residual. ENU current vectors are reconstructed from those projected interface transports for diagnostics, while SST heat advection uses the projected transports directly; ocean diffusion also remains on ocean-only neighbors. SST then feeds back into atmospheric temperature and circulation. WG-5 does not attempt a full 3-D salinity/thermohaline ocean.\n'
replacement = needle + '\nProjected ocean-edge transports drive SST advection through a conservative donor-cell update. Aggregate donor outflow is CFL-limited per orbital phase, so the explicit heat step remains stable as mesh spacing shrinks through the L7 quality target without weakening circulation at coarser levels. Atmospheric moisture transport likewise scales aggregate outgoing graph transfers to the donor water mass before applying paired transfers, preserving moisture mass instead of relying on post-transport zero clamps.\n'
assert needle in doc_text
doc.write_text(doc_text.replace(needle, replacement, 1))
'''
path.write_text(text)
