<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="goblin-miniboss" tilewidth="32" tileheight="32" tilecount="12" columns="3">
 <image source="goblin-miniboss.png" width="96" height="128"/>
 <tile id="3">
  <properties>
   <property name="animation" value="idle"/>
   <property name="loop" type="bool" value="true"/>
  </properties>
  <animation>
   <frame tileid="3" duration="1000"/>
  </animation>
 </tile>
 <tile id="4">
  <properties>
   <property name="animation" value="walk"/>
   <property name="loop" type="bool" value="true"/>
  </properties>
  <animation>
   <frame tileid="4" duration="250"/>
   <frame tileid="3" duration="250"/>
   <frame tileid="5" duration="250"/>
   <frame tileid="3" duration="250"/>
  </animation>
 </tile>
 <tile id="5">
  <properties>
   <property name="animation" value="shoot"/>
   <property name="loop" type="bool" value="false"/>
  </properties>
  <animation>
   <frame tileid="3" duration="1000"/>
  </animation>
 </tile>
 <tile id="8">
  <properties>
   <property name="animation" value="die"/>
   <property name="loop" type="bool" value="true"/>
  </properties>
  <animation>
   <frame tileid="9" duration="1000"/>
  </animation>
 </tile>
 <tile id="10">
  <properties>
   <property name="animation" value="damage"/>
   <property name="loop" type="bool" value="false"/>
  </properties>
  <animation>
   <frame tileid="10" duration="125"/>
   <frame tileid="3" duration="125"/>
  </animation>
 </tile>
</tileset>
