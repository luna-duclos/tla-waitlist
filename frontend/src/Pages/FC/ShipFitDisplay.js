import React from 'react';

export function ShipFitDisplay({ shipTypeId, shipName, items }) {
  // Safety check - if items is not provided or is not an array, show a message
  if (!items || !Array.isArray(items)) {
    return (
      <div style={{ marginBottom: '2em' }}>
        <h4>Ship Fitting</h4>
        <div style={{ padding: '1em', backgroundColor: '#f8f9fa', border: '1px solid #dee2e6', borderRadius: '4px' }}>
          <p>No fitting data available.</p>
        </div>
      </div>
    );
  }

  // Filter out any null/undefined items
  const validItems = items.filter(item => item && typeof item === 'object');

  if (validItems.length === 0) {
    return (
      <div style={{ marginBottom: '2em' }}>
        <h4>Ship Fitting</h4>
        <div style={{ padding: '1em', backgroundColor: '#f8f9fa', border: '1px solid #dee2e6', borderRadius: '4px' }}>
          <p>No valid fitting items found.</p>
        </div>
      </div>
    );
  }

  // Helper function to identify ammo types
  const isAmmoType = (itemName) => {
    if (!itemName) return false;
    const ammoTypes = [
      'Conflagration L', 'Conflagration M', 'Conflagration S',
      'Scorch L', 'Scorch M', 'Scorch S',
      'Radio L', 'Radio M', 'Radio S',
      'Xray L', 'Xray M', 'Xray S',
      'Gamma L', 'Gamma M', 'Gamma S',
      'Multifrequency L', 'Multifrequency M', 'Multifrequency S',
      'Standard L', 'Standard M', 'Standard S',
      'Optimal Range Script', 'Tracking Speed Script'
    ];
    return ammoTypes.includes(itemName);
  };



  // Group items by slot type
  const highSlots = validItems.filter(item => item.flag >= 27 && item.flag <= 34);
  const midSlots = validItems.filter(item => item.flag >= 19 && item.flag <= 26);
  const lowSlots = validItems.filter(item => item.flag >= 11 && item.flag <= 18);
  const rigs = validItems.filter(item => item.flag >= 92 && item.flag <= 99);

  // Separate ammo from modules
  const highSlotModules = highSlots.filter(item => !isAmmoType(item.item_name));
  const highSlotAmmo = highSlots.filter(item => isAmmoType(item.item_name));
  const midSlotModules = midSlots.filter(item => !isAmmoType(item.item_name));
  const midSlotAmmo = midSlots.filter(item => isAmmoType(item.item_name));
  const lowSlotModules = lowSlots.filter(item => !isAmmoType(item.item_name));
  const lowSlotAmmo = lowSlots.filter(item => isAmmoType(item.item_name));

  // Define slot positions (simplified zKillboard positions)
  const getSlotPositions = (slotType) => {
    const positions = {
      high: [
        { left: 73, top: 60 }, { left: 102, top: 42 }, { left: 134, top: 27 },
        { left: 169, top: 21 }, { left: 203, top: 22 }, { left: 238, top: 30 },
        { left: 270, top: 45 }, { left: 295, top: 64 }
      ],
      mid: [
        { left: 26, top: 140 }, { left: 24, top: 176 }, { left: 23, top: 212 },
        { left: 30, top: 245 }, { left: 46, top: 278 }, { left: 69, top: 304 },
        { left: 100, top: 328 }, { left: 133, top: 342 }
      ],
      low: [
        { left: 344, top: 143 }, { left: 350, top: 178 }, { left: 349, top: 213 },
        { left: 340, top: 246 }, { left: 323, top: 277 }, { left: 300, top: 304 },
        { left: 268, top: 324 }, { left: 234, top: 338 }
      ],
      rig: [
        { left: 148, top: 259 }, { left: 185, top: 267 }, { left: 221, top: 259 }
      ]
    };
    return positions[slotType] || [];
  };

  // Define ammo positions relative to their modules
  const getAmmoPositions = (slotType) => {
    const positions = {
      high: [
        { left: 94, top: 88 }, { left: 119, top: 70 }, { left: 146, top: 58 },
        { left: 175, top: 52 }, { left: 204, top: 52 }, { left: 232, top: 60 },
        { left: 258, top: 72 }, { left: 280, top: 91 }
      ],
      mid: [
        { left: 59, top: 154 }, { left: 54, top: 182 }, { left: 56, top: 210 },
        { left: 62, top: 238 }, { left: 76, top: 265 }, { left: 94, top: 288 },
        { left: 118, top: 305 }, { left: 146, top: 318 }
      ],
      low: [
        { left: 315, top: 150 }, { left: 319, top: 179 }, { left: 318, top: 206 },
        { left: 310, top: 234 }, { left: 297, top: 261 }, { left: 275, top: 283 },
        { left: 251, top: 300 }, { left: 225, top: 310 }
      ]
    };
    return positions[slotType] || [];
  };

  const renderSlot = (slotType, modules, ammo, slotPositions, ammoPositions) => {
    return (
      <React.Fragment>
        {/* Render modules */}
        {modules.map((item, index) => {
          const position = slotPositions[index];
          if (!position) return null;
          
          const isDestroyed = item.quantity_destroyed > 0;
          const isDropped = item.quantity_dropped > 0;
          
          return (
            <div
              key={`${slotType}-module-${index}`}
              style={{
                position: 'absolute',
                left: position.left,
                top: position.top,
                width: '32px',
                height: '32px',
                zIndex: 1
              }}
            >
              <img
                src={`https://images.evetech.net/types/${item.item_type_id || 0}/icon?size=32`}
                style={{
                  width: '32px',
                  height: '32px',
                  borderRadius: '4px',
                  opacity: isDestroyed ? 0.5 : 1,
                  border: isDestroyed ? '2px solid #dc3545' : isDropped ? '2px solid #28a745' : '2px solid #dee2e6'
                }}
                alt={item.item_name || 'Unknown Item'}
                title={`${item.item_name || 'Unknown Item'}${isDestroyed ? ` (Destroyed: ${item.quantity_destroyed})` : ''}${isDropped ? ` (Dropped: ${item.quantity_dropped})` : ''}`}
                onError={(e) => {
                  e.target.onerror = null;
                  e.target.src = `https://images.evetech.net/types/${item.item_type_id || 0}/bp?size=32`;
                }}
              />
            </div>
          );
        })}
        
        {/* Render ammo */}
        {ammo.map((item, index) => {
          const position = ammoPositions[index];
          if (!position) return null;
          
          const isDestroyed = item.quantity_destroyed > 0;
          const isDropped = item.quantity_dropped > 0;
          
          return (
            <div
              key={`${slotType}-ammo-${index}`}
              style={{
                position: 'absolute',
                left: position.left,
                top: position.top,
                width: '24px',
                height: '24px',
                zIndex: 2
              }}
            >
              <img
                src={`https://images.evetech.net/types/${item.item_type_id || 0}/icon?size=32`}
                style={{
                  width: '24px',
                  height: '24px',
                  borderRadius: '4px',
                  opacity: isDestroyed ? 0.5 : 1,
                  border: isDestroyed ? '2px solid #dc3545' : isDropped ? '2px solid #28a745' : '2px solid #dee2e6'
                }}
                alt={item.item_name || 'Unknown Item'}
                title={`${item.item_name || 'Unknown Item'}${isDestroyed ? ` (Destroyed: ${item.quantity_destroyed})` : ''}${isDropped ? ` (Dropped: ${item.quantity_dropped})` : ''}`}
                onError={(e) => {
                  e.target.onerror = null;
                  e.target.src = `https://images.evetech.net/types/${item.item_type_id || 0}/bp?size=32`;
                }}
              />
            </div>
          );
        })}
      </React.Fragment>
    );
  };

  return (
    <div style={{ marginBottom: '2em' }}>
      <h4>Ship Fitting</h4>
      <div style={{ 
        backgroundColor: 'white',
        borderRadius: '8px',
        padding: '1.5em',
        border: '1px solid #dee2e6'
      }}>
        {/* Ship Info */}
        <div style={{ marginBottom: '1em', padding: '1em', backgroundColor: '#f8f9fa', borderRadius: '4px' }}>
          <h5 style={{ margin: '0 0 0.5em 0' }}>Ship: {shipName || 'Unknown Ship'}</h5>
          <p style={{ margin: 0, color: '#666' }}>Type ID: {shipTypeId || 'Unknown'}</p>
        </div>

        {/* Fitting Panel */}
        <div style={{ 
          position: 'relative', 
          height: '398px', 
          width: '398px', 
          margin: '0 auto',
          backgroundColor: '#f8f9fa',
          borderRadius: '8px',
          border: '1px solid #dee2e6'
        }}>
          {/* Central Ship Image */}
          <div style={{
            position: 'absolute',
            left: '72px',
            top: '71px',
            width: '256px',
            height: '256px',
            zIndex: -2
          }}>
            <img
              src={`https://images.evetech.net/types/${shipTypeId || 0}/render?size=256`}
              style={{
                height: '256px',
                width: '256px',
                borderRadius: '8px'
              }}
              alt={shipName || 'Unknown Ship'}
              onError={(e) => {
                e.target.onerror = null;
                e.target.src = `https://images.evetech.net/types/${shipTypeId || 0}/icon?size=256`;
              }}
            />
          </div>

          {/* Render all slots */}
          {renderSlot('high', highSlotModules, highSlotAmmo, getSlotPositions('high'), getAmmoPositions('high'))}
          {renderSlot('mid', midSlotModules, midSlotAmmo, getSlotPositions('mid'), getAmmoPositions('mid'))}
          {renderSlot('low', lowSlotModules, lowSlotAmmo, getSlotPositions('low'), getAmmoPositions('low'))}
          {renderSlot('rig', rigs, [], getSlotPositions('rig'), [])}
        </div>

        {/* Legend */}
        <div style={{ 
          marginTop: '1em',
          padding: '0.5em',
          backgroundColor: '#f8f9fa',
          borderRadius: '4px',
          fontSize: '0.9em',
          textAlign: 'center'
        }}>
          <span style={{ color: '#dc3545', marginRight: '1em' }}>✗ Destroyed</span>
          <span style={{ color: '#28a745' }}>✓ Dropped</span>
        </div>
      </div>
    </div>
  );
}
