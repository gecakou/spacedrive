//million-ignore
import { getIcon } from '@sd/assets/util';
import { useLibraryQuery } from '@sd/client';
import React, { useEffect, useRef, useState } from 'react';
import { useIsDark } from '~/hooks';
import ForceGraph2D from 'react-force-graph-2d';
import { useNavigate } from 'react-router';
import * as d3 from 'd3-force';
import * as icons from '../../../../packages/assets/icons';

// NOTE -> tried a lot to keep the nodes within the canvas... including adding d3 forces (basically "rules" to determine node physics/behaviour),
	// replacing the node that's off screen and rerendering the graph (this causes the whole graph to shift over for some reason), and changing the y
	// value of the node directly. it looks like changing the node.y value in the onNodeDragEnd does not mutate the object itself when I console log it...
	// i.e trying to "fix" the y value doesn't actually change the object properly. not sure what else to try and have been pulling my hair out about this!!
interface KindStatistic {
  kind: number;
  name: string;
  count: number;
  total_bytes: string;
}

const FileKindStatistics: React.FC = () => {
  const isDark = useIsDark();
  const navigate = useNavigate();
  const { data } = useLibraryQuery(['library.kindStatistics']);
  const [graphData, setGraphData] = useState({ nodes: [], links: [] });
  const iconsRef = useRef<{ [key: string]: HTMLImageElement }>({});
  const containerRef = useRef<HTMLDivElement>(null);
  const fgRef = useRef<any>(null);

  useEffect(() => {
    if (data) {
      const statistics: KindStatistic[] = data.statistics
        .filter((item: KindStatistic) => item.kind !== 0 && item.count !== 0)
        .sort((a: KindStatistic, b: KindStatistic) => b.count - a.count)
		// TODO: eventually allow users to select and save which file kinds are shown
        .slice(0, 14); // Get the top 14 highest file kinds

      const totalFilesCount = statistics.reduce((sum, item) => sum + item.count, 0);
      const nodes = [
        { id: 'center', name: 'Total Files', val: totalFilesCount },
        ...statistics.map(item => ({
          id: item.kind,
          name: item.name,
          val: item.count,
        }))
      ];

      const links = statistics.map(item => ({
        source: 'center',
        target: item.kind,
      }));

      setGraphData({ nodes, links });

      // Preload icons, this is for rendering purposes
      statistics.forEach(item => {
        const iconName = item.name as keyof typeof icons;
        if (!iconsRef.current[iconName]) {
          const img = new Image();
          img.src = getIcon(iconName, isDark);
          iconsRef.current[iconName] = img;
        }
      });

      // d3 stuff for changing physics of the nodes
      fgRef.current.d3Force('link').distance(80); // Adjust link distance to make links shorter
      fgRef.current.d3Force('charge').strength(-1200); // Adjust charge strength
      fgRef.current.d3Force('collision', d3.forceCollide().radius(19)); // Add collision force with radius. Should be a little larger than radius of nodes.

	  const boundaryForce = () => {
        const width = 1200;
        const height = 100;
        return (alpha: any) => {
          graphData.nodes.forEach((node: any) => {
            node.x = Math.max(-width / 2, Math.min(width / 2, node.x));
            node.y = Math.max(-height / 2, Math.min(height / 2, node.y));
          });
        };
      };

      fgRef.current.d3Force('boundary', boundaryForce());
    }
  }, [data, isDark]);

  const paintNode = (node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
    const fontSize = 0.6 / globalScale;
    ctx.font = `400 ${fontSize}em ui-sans-serif, system-ui, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    const darkColor = 'rgb(34, 34, 45)';
    const lightColor = 'rgb(252, 252, 254)';

    if (node.name === 'Total Files') {
      const radius = 25;
      const borderWidth = 0.5;

      // Create linear gradient for light mode
      const lightGradient = ctx.createLinearGradient(node.x - radius, node.y - radius, node.x + radius, node.y + radius);
      lightGradient.addColorStop(0, 'rgb(117, 177, 249)');
      lightGradient.addColorStop(1, 'rgb(0, 76, 153)');

      // Create linear gradient for dark mode
      const darkGradient = ctx.createLinearGradient(node.x - radius, node.y - radius, node.x + radius, node.y + radius);
      darkGradient.addColorStop(0, 'rgb(255, 13, 202)');
      darkGradient.addColorStop(1, 'rgb(128, 0, 255)');

      // Draw filled circle with gradient border
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI, false);
      ctx.fillStyle = isDark ? darkGradient : lightGradient;
      ctx.fill();

      // Draw inner circle to create the border effect
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius - borderWidth, 0, 2 * Math.PI, false);
      ctx.fillStyle = isDark ? darkColor : lightColor;
      ctx.fill();

      // Add inner shadow
      const shadowGradient = ctx.createRadialGradient(node.x, node.y, radius * 0.5, node.x, node.y, radius);
      shadowGradient.addColorStop(0, 'rgba(0, 0, 0, 0)');
      shadowGradient.addColorStop(1, isDark ? 'rgba(255, 93, 234, 0.1' : 'rgba(66, 97, 255, 0.05)');

      ctx.globalCompositeOperation = 'source-atop';
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI, false);
      ctx.fillStyle = shadowGradient;
      ctx.fill();

      // Draw text
      ctx.fillStyle = isDark ? 'rgba(255, 255, 255, 1)' : 'rgba(10, 10, 10, 0.8)';
      ctx.font = `bold ${fontSize * 2}em ui-sans-serif, system-ui, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"`;
      ctx.fillText(node.val, node.x, node.y - fontSize * 9);

      ctx.fillStyle = isDark ? 'rgba(255, 255, 255, 0.3)' : 'rgba(10, 10, 10, 0.8)';
      ctx.font = `400 ${fontSize * 1.1}em ui-sans-serif, system-ui, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"`;
      ctx.fillText(node.name, node.x, node.y + fontSize * 25);
    } else {
      const iconName = node.name as keyof typeof icons;
      const iconImg = iconsRef.current[iconName];
      const iconSize = 25 / globalScale;
      const textYPos = node.y + iconSize;

      // Draw shadow
      ctx.shadowColor = isDark ? 'rgb(44, 45, 58)' : 'rgba(0, 0, 0, 0.1)';
      ctx.shadowBlur = 0.5;
      ctx.shadowOffsetX = -0.5;
      ctx.shadowOffsetY = -2;

      // Draw node circle
      const radius = 18;
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI, false);
      ctx.fillStyle = isDark ? darkColor : lightColor;
      ctx.fill();
      ctx.shadowColor = 'transparent';

      if (iconImg) {
        ctx.drawImage(iconImg, node.x - iconSize / 2, node.y - iconSize, iconSize, iconSize);
      }

      ctx.fillStyle = isDark ? 'white' : 'black';

      // Truncate node name if it is too long
      let truncatedName = node.name;
      if (node.name.length > 10) {
        truncatedName = node.name.slice(0, 6) + "...";
      }
      ctx.fillText(truncatedName, node.x, textYPos - 9);

      ctx.fillStyle = isDark ? 'rgba(255, 255, 255, 0.3)' : 'rgba(0, 0, 0, 0.5)';
      ctx.fillText(node.val, node.x, textYPos - 2);
    }
  };

  const handleNodeClick = (node: any) => {
    if (node.id !== 'center') {
      const path = {
        pathname: '../search',
        search: new URLSearchParams({
          filters: JSON.stringify([{ object: { kind: { in: [node.id] } } }])
        }).toString()
      };
      navigate(path);
    }
  };

  const replaceNodeIfOutOfBound = (node: any, translation: any) => {
	console.log('Before:', node);

	// Directly update the node's y value
	node.y = 100;
	console.log('After:', node);
  };

  const paintPointerArea = (node: any, color: string, ctx: CanvasRenderingContext2D, globalScale: number) => {
    const size = 30 / globalScale; // Adjust this size to match the node size
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(node.x, node.y, size, 0, 2 * Math.PI, false);
    ctx.fill();
  };
  return (
    <div className="relative bottom-24 right-36 h-[200px] w-full" ref={containerRef}>
      {data ? (
        <ForceGraph2D
          ref={fgRef}
          graphData={graphData}
          nodeId="id"
          linkSource="source"
          linkTarget="target"
          width={1200}
          height={400}
          backgroundColor="transparent"
          nodeCanvasObject={paintNode}
          linkWidth={0.5}
          nodeLabel=""
		  dagMode="td"
          linkColor={() => isDark ? '#2C2D3A' : 'rgba(0, 0, 0, 0.2)'}
          onNodeClick={handleNodeClick}
          onNodeDragEnd={(node, translation) => replaceNodeIfOutOfBound(node, translation)}
          enableZoomInteraction={false}
          enablePanInteraction={false}
          dagLevelDistance={60}
          warmupTicks={300}
          nodePointerAreaPaint={paintPointerArea}
        />
      ) : (
        <div>Loading...</div>
      )}
    </div>
  );
};

export default FileKindStatistics;
