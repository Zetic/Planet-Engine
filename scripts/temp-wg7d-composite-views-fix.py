from pathlib import Path

path = Path('index.html')
text = path.read_text()
text = text.replace('data-label="WG-4 topographic contours"> WG-4 topographic contours', 'data-label="WG-4 Topographic contours"> WG-4 Topographic contours')
path.write_text(text)
